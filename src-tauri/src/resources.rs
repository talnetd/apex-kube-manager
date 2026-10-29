//! Extra resource types (spec 12): RBAC, StorageClasses, EndpointSlices,
//! IngressClasses, PDBs, ResourceQuotas, LimitRanges, PriorityClasses, CRDs
//! and custom resources.
//!
//! Lists are typed (one row struct per kind, matching its table columns).
//! Detail, YAML and events go through one dispatch on the frontend kind id,
//! returning a generic detail shape (key/value fields plus small tables)
//! that `ResourceDetail.svelte` renders for every kind here, so a new type
//! needs a list function and a detail builder, not a new screen.

use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::{Event, LimitRange, ResourceQuota};
use k8s_openapi::api::discovery::v1::EndpointSlice;
use k8s_openapi::api::networking::v1::IngressClass;
use k8s_openapi::api::policy::v1::PodDisruptionBudget;
use k8s_openapi::api::rbac::v1::{
    ClusterRole, ClusterRoleBinding, PolicyRule, Role, RoleBinding, RoleRef, Subject,
};
use k8s_openapi::api::scheduling::v1::PriorityClass;
use k8s_openapi::api::storage::v1::StorageClass;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{Api, ApiResource, DynamicObject, ListParams};
use kube::Client;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::kubernetes::{get_age, TimestampExt};

// ============ Generic detail shape ============

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailField {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailTable {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericDetail {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: BTreeMap<String, String>,
    pub annotations: BTreeMap<String, String>,
    pub fields: Vec<DetailField>,
    pub tables: Vec<DetailTable>,
}

/// Same shape as the other `*Event` types, so `EventsTable` renders it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

fn base(kind: &str, meta: &ObjectMeta) -> GenericDetail {
    GenericDetail {
        kind: kind.to_string(),
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone(),
        uid: meta.uid.clone().unwrap_or_default(),
        creation_timestamp: meta
            .creation_timestamp
            .as_ref()
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: meta.labels.clone().unwrap_or_default(),
        annotations: meta.annotations.clone().unwrap_or_default(),
        fields: Vec::new(),
        tables: Vec::new(),
    }
}

impl GenericDetail {
    fn field(mut self, label: &str, value: impl Into<String>) -> Self {
        let value = value.into();
        self.fields.push(DetailField {
            label: label.to_string(),
            value: if value.is_empty() {
                "–".to_string()
            } else {
                value
            },
        });
        self
    }

    fn table(mut self, title: &str, columns: &[&str], rows: Vec<Vec<String>>) -> Self {
        if !rows.is_empty() {
            self.tables.push(DetailTable {
                title: title.to_string(),
                columns: columns.iter().map(|c| c.to_string()).collect(),
                rows,
            });
        }
        self
    }
}

// ============ Small formatting helpers (unit-tested below) ============

fn join(v: &Option<Vec<String>>) -> String {
    match v {
        Some(v) if !v.is_empty() => v.join(", "),
        _ => String::new(),
    }
}

fn yes_no(b: bool) -> String {
    if b { "Yes" } else { "No" }.to_string()
}

fn int_or_string(v: &Option<IntOrString>) -> String {
    match v {
        Some(IntOrString::Int(i)) => i.to_string(),
        Some(IntOrString::String(s)) => s.clone(),
        None => String::new(),
    }
}

/// `app=web, tier in (api,db)`; empty for a missing selector, `<all>` for `{}`.
pub fn selector_string(sel: &Option<LabelSelector>) -> String {
    let Some(sel) = sel else { return String::new() };
    let mut parts: Vec<String> = sel
        .match_labels
        .as_ref()
        .map(|m| m.iter().map(|(k, v)| format!("{k}={v}")).collect())
        .unwrap_or_default();
    for e in sel.match_expressions.iter().flatten() {
        let values = e.values.clone().unwrap_or_default().join(",");
        parts.push(match e.operator.as_str() {
            "Exists" => e.key.clone(),
            "DoesNotExist" => format!("!{}", e.key),
            op => format!("{} {} ({})", e.key, op.to_lowercase(), values),
        });
    }
    if parts.is_empty() {
        "<all>".to_string()
    } else {
        parts.join(", ")
    }
}

fn quantities(m: &Option<BTreeMap<String, Quantity>>) -> BTreeMap<String, String> {
    m.as_ref()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect())
        .unwrap_or_default()
}

/// One-line subject summary for binding lists: `User alice, Group devs +2`.
pub fn subjects_summary(subjects: &Option<Vec<Subject>>) -> String {
    let subjects = subjects.as_deref().unwrap_or_default();
    let shown: Vec<String> = subjects
        .iter()
        .take(2)
        .map(|s| match &s.namespace {
            Some(ns) if s.kind == "ServiceAccount" => format!("{} {}/{}", s.kind, ns, s.name),
            _ => format!("{} {}", s.kind, s.name),
        })
        .collect();
    let rest = subjects.len().saturating_sub(shown.len());
    let mut out = shown.join(", ");
    if rest > 0 {
        out.push_str(&format!(" +{rest}"));
    }
    out
}

fn role_ref_string(r: &RoleRef) -> String {
    format!("{}/{}", r.kind, r.name)
}

fn rule_rows(rules: &Option<Vec<PolicyRule>>) -> Vec<Vec<String>> {
    rules
        .iter()
        .flatten()
        .map(|r| {
            let resources = match (&r.resources, &r.non_resource_urls) {
                (Some(res), _) if !res.is_empty() => res.join(", "),
                (_, Some(urls)) => urls.join(", "),
                _ => String::new(),
            };
            let groups = match &r.api_groups {
                Some(g) if !g.is_empty() => g
                    .iter()
                    .map(|g| {
                        if g.is_empty() {
                            "core".to_string()
                        } else {
                            g.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => String::new(),
            };
            vec![
                groups,
                resources,
                r.verbs.join(", "),
                join(&r.resource_names),
            ]
        })
        .collect()
}

fn subject_rows(subjects: &Option<Vec<Subject>>) -> Vec<Vec<String>> {
    subjects
        .iter()
        .flatten()
        .map(|s| {
            vec![
                s.kind.clone(),
                s.name.clone(),
                s.namespace.clone().unwrap_or_default(),
            ]
        })
        .collect()
}

fn annotation_true(meta: &ObjectMeta, key: &str) -> bool {
    meta.annotations
        .as_ref()
        .and_then(|a| a.get(key))
        .is_some_and(|v| v == "true")
}

const DEFAULT_SC: &str = "storageclass.kubernetes.io/is-default-class";
const DEFAULT_IC: &str = "ingressclass.kubernetes.io/is-default-class";
const SLICE_SERVICE: &str = "kubernetes.io/service-name";

// ============ List rows ============

fn ns_api<K>(client: &Client, namespace: Option<&str>) -> Api<K>
where
    K: kube::Resource<Scope = k8s_openapi::NamespaceResourceScope>,
    <K as kube::Resource>::DynamicType: Default,
{
    match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    }
}

fn name_of(m: &ObjectMeta) -> String {
    m.name.clone().unwrap_or_default()
}
fn ns_of(m: &ObjectMeta) -> String {
    m.namespace.clone().unwrap_or_default()
}
fn age_of(m: &ObjectMeta) -> String {
    get_age(m.creation_timestamp.as_ref())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub name: String,
    pub namespace: String,
    pub rules: usize,
    pub age: String,
}

pub async fn list_roles(client: &Client, namespace: Option<&str>) -> Result<Vec<RoleInfo>> {
    let list = ns_api::<Role>(client, namespace)
        .list(&ListParams::default())
        .await?;
    Ok(list
        .items
        .iter()
        .map(|r| RoleInfo {
            name: name_of(&r.metadata),
            namespace: ns_of(&r.metadata),
            rules: r.rules.as_ref().map_or(0, |v| v.len()),
            age: age_of(&r.metadata),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRoleInfo {
    pub name: String,
    pub rules: usize,
    pub aggregated: bool,
    pub age: String,
}

pub async fn list_cluster_roles(client: &Client) -> Result<Vec<ClusterRoleInfo>> {
    let api: Api<ClusterRole> = Api::all(client.clone());
    let list = api.list(&ListParams::default()).await?;
    Ok(list
        .items
        .iter()
        .map(|r| ClusterRoleInfo {
            name: name_of(&r.metadata),
            rules: r.rules.as_ref().map_or(0, |v| v.len()),
            aggregated: r.aggregation_rule.is_some(),
            age: age_of(&r.metadata),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBindingInfo {
    pub name: String,
    pub namespace: String,
    pub role_ref: String,
    pub subjects: String,
    pub subject_count: usize,
    pub age: String,
}

pub async fn list_role_bindings(
    client: &Client,
    namespace: Option<&str>,
) -> Result<Vec<RoleBindingInfo>> {
    let list = ns_api::<RoleBinding>(client, namespace)
        .list(&ListParams::default())
        .await?;
    Ok(list
        .items
        .iter()
        .map(|b| RoleBindingInfo {
            name: name_of(&b.metadata),
            namespace: ns_of(&b.metadata),
            role_ref: role_ref_string(&b.role_ref),
            subjects: subjects_summary(&b.subjects),
            subject_count: b.subjects.as_ref().map_or(0, |v| v.len()),
            age: age_of(&b.metadata),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRoleBindingInfo {
    pub name: String,
    pub role_ref: String,
    pub subjects: String,
    pub subject_count: usize,
    pub age: String,
}

pub async fn list_cluster_role_bindings(client: &Client) -> Result<Vec<ClusterRoleBindingInfo>> {
    let api: Api<ClusterRoleBinding> = Api::all(client.clone());
    let list = api.list(&ListParams::default()).await?;
    Ok(list
        .items
        .iter()
        .map(|b| ClusterRoleBindingInfo {
            name: name_of(&b.metadata),
            role_ref: role_ref_string(&b.role_ref),
            subjects: subjects_summary(&b.subjects),
            subject_count: b.subjects.as_ref().map_or(0, |v| v.len()),
            age: age_of(&b.metadata),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClassInfo {
    pub name: String,
    pub provisioner: String,
    pub reclaim_policy: String,
    pub binding_mode: String,
    pub allow_expansion: bool,
    pub is_default: bool,
    pub age: String,
}

pub async fn list_storage_classes(client: &Client) -> Result<Vec<StorageClassInfo>> {
    let api: Api<StorageClass> = Api::all(client.clone());
    let list = api.list(&ListParams::default()).await?;
    Ok(list
        .items
        .iter()
        .map(|sc| StorageClassInfo {
            name: name_of(&sc.metadata),
            provisioner: sc.provisioner.clone(),
            reclaim_policy: sc.reclaim_policy.clone().unwrap_or_else(|| "Delete".into()),
            binding_mode: sc
                .volume_binding_mode
                .clone()
                .unwrap_or_else(|| "Immediate".into()),
            allow_expansion: sc.allow_volume_expansion.unwrap_or(false),
            is_default: annotation_true(&sc.metadata, DEFAULT_SC),
            age: age_of(&sc.metadata),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSliceInfo {
    pub name: String,
    pub namespace: String,
    pub service: String,
    pub address_type: String,
    pub ready: usize,
    pub total: usize,
    pub ports: String,
    pub age: String,
}

/// An endpoint counts as ready unless its condition says otherwise (the
/// EndpointSlice API treats a missing `ready` as ready).
pub fn slice_ready(slice: &EndpointSlice) -> (usize, usize) {
    let eps = slice.endpoints.as_deref().unwrap_or_default();
    let ready = eps
        .iter()
        .filter(|e| e.conditions.as_ref().and_then(|c| c.ready).unwrap_or(true))
        .count();
    (ready, eps.len())
}

fn slice_ports(slice: &EndpointSlice) -> String {
    slice
        .ports
        .iter()
        .flatten()
        .map(|p| {
            let port = p.port.map(|n| n.to_string()).unwrap_or_else(|| "*".into());
            let proto = p.protocol.clone().unwrap_or_else(|| "TCP".into());
            match &p.name {
                Some(n) if !n.is_empty() => format!("{n} {port}/{proto}"),
                _ => format!("{port}/{proto}"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub async fn list_endpoint_slices(
    client: &Client,
    namespace: Option<&str>,
) -> Result<Vec<EndpointSliceInfo>> {
    let list = ns_api::<EndpointSlice>(client, namespace)
        .list(&ListParams::default())
        .await?;
    Ok(list
        .items
        .iter()
        .map(|s| {
            let (ready, total) = slice_ready(s);
            EndpointSliceInfo {
                name: name_of(&s.metadata),
                namespace: ns_of(&s.metadata),
                service: s
                    .metadata
                    .labels
                    .as_ref()
                    .and_then(|l| l.get(SLICE_SERVICE))
                    .cloned()
                    .unwrap_or_default(),
                address_type: s.address_type.clone(),
                ready,
                total,
                ports: slice_ports(s),
                age: age_of(&s.metadata),
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressClassInfo {
    pub name: String,
    pub controller: String,
    pub is_default: bool,
    pub parameters: String,
    pub age: String,
}

pub async fn list_ingress_classes(client: &Client) -> Result<Vec<IngressClassInfo>> {
    let api: Api<IngressClass> = Api::all(client.clone());
    let list = api.list(&ListParams::default()).await?;
    Ok(list
        .items
        .iter()
        .map(|ic| {
            let spec = ic.spec.as_ref();
            IngressClassInfo {
                name: name_of(&ic.metadata),
                controller: spec.and_then(|s| s.controller.clone()).unwrap_or_default(),
                is_default: annotation_true(&ic.metadata, DEFAULT_IC),
                parameters: spec
                    .and_then(|s| s.parameters.as_ref())
                    .map(|p| format!("{}/{}", p.kind, p.name))
                    .unwrap_or_default(),
                age: age_of(&ic.metadata),
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdbInfo {
    pub name: String,
    pub namespace: String,
    pub min_available: String,
    pub max_unavailable: String,
    pub allowed_disruptions: i32,
    pub current_healthy: i32,
    pub desired_healthy: i32,
    pub expected_pods: i32,
    pub age: String,
}

pub async fn list_pdbs(client: &Client, namespace: Option<&str>) -> Result<Vec<PdbInfo>> {
    let list = ns_api::<PodDisruptionBudget>(client, namespace)
        .list(&ListParams::default())
        .await?;
    Ok(list
        .items
        .iter()
        .map(|p| {
            let spec = p.spec.as_ref();
            let st = p.status.as_ref();
            PdbInfo {
                name: name_of(&p.metadata),
                namespace: ns_of(&p.metadata),
                min_available: int_or_string(&spec.and_then(|s| s.min_available.clone())),
                max_unavailable: int_or_string(&spec.and_then(|s| s.max_unavailable.clone())),
                allowed_disruptions: st.and_then(|s| s.disruptions_allowed).unwrap_or(0),
                current_healthy: st.and_then(|s| s.current_healthy).unwrap_or(0),
                desired_healthy: st.and_then(|s| s.desired_healthy).unwrap_or(0),
                expected_pods: st.and_then(|s| s.expected_pods).unwrap_or(0),
                age: age_of(&p.metadata),
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuotaInfo {
    pub name: String,
    pub namespace: String,
    /// Raw quantities; the frontend works out the percentages.
    pub hard: BTreeMap<String, String>,
    pub used: BTreeMap<String, String>,
    pub age: String,
}

pub async fn list_resource_quotas(
    client: &Client,
    namespace: Option<&str>,
) -> Result<Vec<ResourceQuotaInfo>> {
    let list = ns_api::<ResourceQuota>(client, namespace)
        .list(&ListParams::default())
        .await?;
    Ok(list
        .items
        .iter()
        .map(|q| {
            let st = q.status.as_ref();
            let hard = st
                .map(|s| quantities(&s.hard))
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| quantities(&q.spec.as_ref().and_then(|s| s.hard.clone())));
            ResourceQuotaInfo {
                name: name_of(&q.metadata),
                namespace: ns_of(&q.metadata),
                hard,
                used: st.map(|s| quantities(&s.used)).unwrap_or_default(),
                age: age_of(&q.metadata),
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitRangeInfo {
    pub name: String,
    pub namespace: String,
    pub types: String,
    pub limits: usize,
    pub age: String,
}

pub async fn list_limit_ranges(
    client: &Client,
    namespace: Option<&str>,
) -> Result<Vec<LimitRangeInfo>> {
    let list = ns_api::<LimitRange>(client, namespace)
        .list(&ListParams::default())
        .await?;
    Ok(list
        .items
        .iter()
        .map(|l| {
            let items = l
                .spec
                .as_ref()
                .map(|s| s.limits.as_slice())
                .unwrap_or_default();
            let mut types: Vec<String> = items.iter().map(|i| i.type_.clone()).collect();
            types.dedup();
            LimitRangeInfo {
                name: name_of(&l.metadata),
                namespace: ns_of(&l.metadata),
                types: types.join(", "),
                limits: items.len(),
                age: age_of(&l.metadata),
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityClassInfo {
    pub name: String,
    pub value: i32,
    pub global_default: bool,
    pub preemption_policy: String,
    pub description: String,
    pub age: String,
}

pub async fn list_priority_classes(client: &Client) -> Result<Vec<PriorityClassInfo>> {
    let api: Api<PriorityClass> = Api::all(client.clone());
    let list = api.list(&ListParams::default()).await?;
    Ok(list
        .items
        .iter()
        .map(|p| PriorityClassInfo {
            name: name_of(&p.metadata),
            value: p.value.unwrap_or(0),
            global_default: p.global_default.unwrap_or(false),
            preemption_policy: p
                .preemption_policy
                .clone()
                .unwrap_or_else(|| "PreemptLowerPriority".into()),
            description: p.description.clone().unwrap_or_default(),
            age: age_of(&p.metadata),
        })
        .collect())
}

// ============ CRDs and custom resources ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdInfo {
    pub name: String,
    pub group: String,
    pub kind: String,
    pub plural: String,
    pub scope: String,
    pub versions: Vec<String>,
    /// The version to list instances with: the storage version if served,
    /// else the first served one.
    pub version: String,
    pub age: String,
}

pub fn preferred_version(crd: &CustomResourceDefinition) -> String {
    let vs = &crd.spec.versions;
    vs.iter()
        .find(|v| v.storage && v.served)
        .or_else(|| vs.iter().find(|v| v.served))
        .or_else(|| vs.first())
        .map(|v| v.name.clone())
        .unwrap_or_default()
}

pub async fn list_crds(client: &Client) -> Result<Vec<CrdInfo>> {
    let api: Api<CustomResourceDefinition> = Api::all(client.clone());
    let list = api.list(&ListParams::default()).await?;
    Ok(list
        .items
        .iter()
        .map(|c| CrdInfo {
            name: name_of(&c.metadata),
            group: c.spec.group.clone(),
            kind: c.spec.names.kind.clone(),
            plural: c.spec.names.plural.clone(),
            scope: c.spec.scope.clone(),
            versions: c
                .spec
                .versions
                .iter()
                .filter(|v| v.served)
                .map(|v| v.name.clone())
                .collect(),
            version: preferred_version(c),
            age: age_of(&c.metadata),
        })
        .collect())
}

/// Identifies one custom resource type for the dynamic API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTarget {
    pub group: String,
    pub version: String,
    pub kind: String,
    pub plural: String,
    pub namespaced: bool,
}

impl CustomTarget {
    fn api_resource(&self) -> ApiResource {
        ApiResource {
            group: self.group.clone(),
            version: self.version.clone(),
            api_version: if self.group.is_empty() {
                self.version.clone()
            } else {
                format!("{}/{}", self.group, self.version)
            },
            kind: self.kind.clone(),
            plural: self.plural.clone(),
        }
    }

    fn api(&self, client: &Client, namespace: Option<&str>) -> Api<DynamicObject> {
        let ar = self.api_resource();
        match (self.namespaced, namespace) {
            (true, Some(ns)) => Api::namespaced_with(client.clone(), ns, &ar),
            _ => Api::all_with(client.clone(), &ar),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomResourceInfo {
    pub name: String,
    pub namespace: String,
    pub age: String,
}

pub async fn list_custom_resources(
    client: &Client,
    target: &CustomTarget,
    namespace: Option<&str>,
) -> Result<Vec<CustomResourceInfo>> {
    let list = target
        .api(client, namespace)
        .list(&ListParams::default())
        .await?;
    Ok(list
        .items
        .iter()
        .map(|o| CustomResourceInfo {
            name: name_of(&o.metadata),
            namespace: ns_of(&o.metadata),
            age: age_of(&o.metadata),
        })
        .collect())
}

fn custom_detail(target: &CustomTarget, obj: &DynamicObject) -> GenericDetail {
    let mut d = base(&target.kind, &obj.metadata)
        .field("API version", target.api_resource().api_version)
        .field("Kind", target.kind.clone());
    // Top-level spec and status keys hint at what the object holds; the YAML
    // tab has the full content.
    for section in ["spec", "status"] {
        if let Some(map) = obj.data.get(section).and_then(|v| v.as_object()) {
            let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            let label = if section == "spec" {
                "Spec fields"
            } else {
                "Status fields"
            };
            d = d.field(label, keys.join(", "));
        }
    }
    d
}

pub async fn custom_resource_detail(
    client: &Client,
    target: &CustomTarget,
    namespace: Option<&str>,
    name: &str,
) -> Result<GenericDetail> {
    let obj = target.api(client, namespace).get(name).await?;
    Ok(custom_detail(target, &obj))
}

pub async fn custom_resource_yaml(
    client: &Client,
    target: &CustomTarget,
    namespace: Option<&str>,
    name: &str,
) -> Result<String> {
    let obj = target.api(client, namespace).get(name).await?;
    Ok(serde_yaml::to_string(&obj)?)
}

// ============ Detail builders ============

fn role_detail(r: &Role) -> GenericDetail {
    base("Role", &r.metadata)
        .field("Rules", r.rules.as_ref().map_or(0, |v| v.len()).to_string())
        .table(
            "Rules",
            &["API groups", "Resources", "Verbs", "Resource names"],
            rule_rows(&r.rules),
        )
}

fn cluster_role_detail(r: &ClusterRole) -> GenericDetail {
    let aggregation = r
        .aggregation_rule
        .as_ref()
        .and_then(|a| a.cluster_role_selectors.as_ref())
        .map(|sels| {
            sels.iter()
                .map(|s| selector_string(&Some(s.clone())))
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default();
    base("ClusterRole", &r.metadata)
        .field("Rules", r.rules.as_ref().map_or(0, |v| v.len()).to_string())
        .field("Aggregates", aggregation)
        .table(
            "Rules",
            &["API groups", "Resources", "Verbs", "Resource names"],
            rule_rows(&r.rules),
        )
}

fn role_binding_detail(b: &RoleBinding) -> GenericDetail {
    base("RoleBinding", &b.metadata)
        .field("Role", role_ref_string(&b.role_ref))
        .field(
            "Subjects",
            b.subjects.as_ref().map_or(0, |v| v.len()).to_string(),
        )
        .table(
            "Subjects",
            &["Kind", "Name", "Namespace"],
            subject_rows(&b.subjects),
        )
}

fn cluster_role_binding_detail(b: &ClusterRoleBinding) -> GenericDetail {
    base("ClusterRoleBinding", &b.metadata)
        .field("Role", role_ref_string(&b.role_ref))
        .field(
            "Subjects",
            b.subjects.as_ref().map_or(0, |v| v.len()).to_string(),
        )
        .table(
            "Subjects",
            &["Kind", "Name", "Namespace"],
            subject_rows(&b.subjects),
        )
}

fn storage_class_detail(sc: &StorageClass) -> GenericDetail {
    let params = sc
        .parameters
        .iter()
        .flatten()
        .map(|(k, v)| vec![k.clone(), v.clone()])
        .collect();
    base("StorageClass", &sc.metadata)
        .field("Provisioner", sc.provisioner.clone())
        .field(
            "Reclaim policy",
            sc.reclaim_policy.clone().unwrap_or_else(|| "Delete".into()),
        )
        .field(
            "Binding mode",
            sc.volume_binding_mode
                .clone()
                .unwrap_or_else(|| "Immediate".into()),
        )
        .field(
            "Volume expansion",
            yes_no(sc.allow_volume_expansion.unwrap_or(false)),
        )
        .field(
            "Default class",
            yes_no(annotation_true(&sc.metadata, DEFAULT_SC)),
        )
        .field("Mount options", join(&sc.mount_options))
        .table("Parameters", &["Key", "Value"], params)
}

fn endpoint_slice_detail(s: &EndpointSlice) -> GenericDetail {
    let (ready, total) = slice_ready(s);
    let endpoints = s
        .endpoints
        .iter()
        .flatten()
        .map(|e| {
            let ready = e.conditions.as_ref().and_then(|c| c.ready).unwrap_or(true);
            let target = e
                .target_ref
                .as_ref()
                .map(|t| {
                    format!(
                        "{}/{}",
                        t.kind.clone().unwrap_or_default(),
                        t.name.clone().unwrap_or_default()
                    )
                })
                .unwrap_or_default();
            vec![
                e.addresses.join(", "),
                yes_no(ready),
                e.node_name.clone().unwrap_or_default(),
                target,
            ]
        })
        .collect();
    let ports = s
        .ports
        .iter()
        .flatten()
        .map(|p| {
            vec![
                p.name.clone().unwrap_or_default(),
                p.port.map(|n| n.to_string()).unwrap_or_default(),
                p.protocol.clone().unwrap_or_else(|| "TCP".into()),
            ]
        })
        .collect();
    base("EndpointSlice", &s.metadata)
        .field(
            "Service",
            s.metadata
                .labels
                .as_ref()
                .and_then(|l| l.get(SLICE_SERVICE))
                .cloned()
                .unwrap_or_default(),
        )
        .field("Address type", s.address_type.clone())
        .field("Ready", format!("{ready}/{total}"))
        .table(
            "Endpoints",
            &["Addresses", "Ready", "Node", "Target"],
            endpoints,
        )
        .table("Ports", &["Name", "Port", "Protocol"], ports)
}

fn ingress_class_detail(ic: &IngressClass) -> GenericDetail {
    let spec = ic.spec.as_ref();
    base("IngressClass", &ic.metadata)
        .field(
            "Controller",
            spec.and_then(|s| s.controller.clone()).unwrap_or_default(),
        )
        .field(
            "Default class",
            yes_no(annotation_true(&ic.metadata, DEFAULT_IC)),
        )
        .field(
            "Parameters",
            spec.and_then(|s| s.parameters.as_ref())
                .map(|p| format!("{}/{}", p.kind, p.name))
                .unwrap_or_default(),
        )
}

fn pdb_detail(p: &PodDisruptionBudget) -> GenericDetail {
    let spec = p.spec.as_ref();
    let st = p.status.as_ref();
    let n = |v: Option<i32>| v.unwrap_or(0).to_string();
    let conditions = st
        .and_then(|s| s.conditions.as_ref())
        .into_iter()
        .flatten()
        .map(|c| {
            vec![
                c.type_.clone(),
                c.status.clone(),
                c.reason.clone(),
                c.message.clone(),
            ]
        })
        .collect();
    base("PodDisruptionBudget", &p.metadata)
        .field(
            "Min available",
            int_or_string(&spec.and_then(|s| s.min_available.clone())),
        )
        .field(
            "Max unavailable",
            int_or_string(&spec.and_then(|s| s.max_unavailable.clone())),
        )
        .field(
            "Allowed disruptions",
            n(st.and_then(|s| s.disruptions_allowed)),
        )
        .field("Current healthy", n(st.and_then(|s| s.current_healthy)))
        .field("Desired healthy", n(st.and_then(|s| s.desired_healthy)))
        .field("Expected pods", n(st.and_then(|s| s.expected_pods)))
        .field(
            "Selector",
            selector_string(&spec.and_then(|s| s.selector.clone())),
        )
        .field(
            "Unhealthy pod eviction",
            spec.and_then(|s| s.unhealthy_pod_eviction_policy.clone())
                .unwrap_or_else(|| "IfHealthyBudget".into()),
        )
        .table(
            "Conditions",
            &["Type", "Status", "Reason", "Message"],
            conditions,
        )
}

fn resource_quota_detail(q: &ResourceQuota) -> GenericDetail {
    let st = q.status.as_ref();
    let hard = st.map(|s| quantities(&s.hard)).unwrap_or_default();
    let used = st.map(|s| quantities(&s.used)).unwrap_or_default();
    let rows = hard
        .iter()
        .map(|(k, h)| {
            vec![
                k.clone(),
                used.get(k).cloned().unwrap_or_else(|| "0".into()),
                h.clone(),
            ]
        })
        .collect();
    base("ResourceQuota", &q.metadata)
        .field(
            "Scopes",
            join(&q.spec.as_ref().and_then(|s| s.scopes.clone())),
        )
        .table("Usage", &["Resource", "Used", "Hard"], rows)
}

fn limit_range_detail(l: &LimitRange) -> GenericDetail {
    let mut rows = Vec::new();
    for item in l
        .spec
        .as_ref()
        .map(|s| s.limits.as_slice())
        .unwrap_or_default()
    {
        let maps = [
            quantities(&item.min),
            quantities(&item.max),
            quantities(&item.default),
            quantities(&item.default_request),
            quantities(&item.max_limit_request_ratio),
        ];
        let mut resources: Vec<&String> = maps.iter().flat_map(|m| m.keys()).collect();
        resources.sort();
        resources.dedup();
        for r in resources {
            let get = |m: &BTreeMap<String, String>| m.get(r).cloned().unwrap_or_default();
            rows.push(vec![
                item.type_.clone(),
                r.clone(),
                get(&maps[0]),
                get(&maps[1]),
                get(&maps[2]),
                get(&maps[3]),
                get(&maps[4]),
            ]);
        }
    }
    base("LimitRange", &l.metadata).table(
        "Limits",
        &[
            "Type",
            "Resource",
            "Min",
            "Max",
            "Default",
            "Default request",
            "Max ratio",
        ],
        rows,
    )
}

fn priority_class_detail(p: &PriorityClass) -> GenericDetail {
    base("PriorityClass", &p.metadata)
        .field("Value", p.value.unwrap_or(0).to_string())
        .field("Global default", yes_no(p.global_default.unwrap_or(false)))
        .field(
            "Preemption policy",
            p.preemption_policy
                .clone()
                .unwrap_or_else(|| "PreemptLowerPriority".into()),
        )
        .field("Description", p.description.clone().unwrap_or_default())
}

fn crd_detail(c: &CustomResourceDefinition) -> GenericDetail {
    let versions = c
        .spec
        .versions
        .iter()
        .map(|v| {
            vec![
                v.name.clone(),
                yes_no(v.served),
                yes_no(v.storage),
                yes_no(v.deprecated.unwrap_or(false)),
            ]
        })
        .collect();
    base("CustomResourceDefinition", &c.metadata)
        .field("Group", c.spec.group.clone())
        .field("Kind", c.spec.names.kind.clone())
        .field("Plural", c.spec.names.plural.clone())
        .field("Scope", c.spec.scope.clone())
        .field("Short names", join(&c.spec.names.short_names))
        .table(
            "Versions",
            &["Name", "Served", "Storage", "Deprecated"],
            versions,
        )
}

// ============ Dispatch on the frontend kind id ============

/// Frontend kind id -> (API kind for event lookups, namespaced).
pub fn kind_info(kind: &str) -> Option<(&'static str, bool)> {
    Some(match kind {
        "role" => ("Role", true),
        "clusterrole" => ("ClusterRole", false),
        "rolebinding" => ("RoleBinding", true),
        "clusterrolebinding" => ("ClusterRoleBinding", false),
        "storageclass" => ("StorageClass", false),
        "endpointslice" => ("EndpointSlice", true),
        "ingressclass" => ("IngressClass", false),
        "pdb" => ("PodDisruptionBudget", true),
        "resourcequota" => ("ResourceQuota", true),
        "limitrange" => ("LimitRange", true),
        "priorityclass" => ("PriorityClass", false),
        "crd" => ("CustomResourceDefinition", false),
        _ => return None,
    })
}

fn unknown(kind: &str) -> AppError {
    AppError::Custom(format!("Unknown resource kind: {kind}"))
}

fn need_ns(namespace: Option<&str>) -> Result<&str> {
    namespace
        .filter(|n| !n.is_empty())
        .ok_or_else(|| AppError::Custom("A namespace is required for this resource".into()))
}

macro_rules! fetch_ns {
    ($t:ty, $client:expr, $ns:expr, $name:expr) => {{
        let api: Api<$t> = Api::namespaced($client.clone(), need_ns($ns)?);
        api.get($name).await?
    }};
}

macro_rules! fetch_all {
    ($t:ty, $client:expr, $name:expr) => {{
        let api: Api<$t> = Api::all($client.clone());
        api.get($name).await?
    }};
}

pub async fn resource_detail(
    client: &Client,
    kind: &str,
    namespace: Option<&str>,
    name: &str,
) -> Result<GenericDetail> {
    Ok(match kind {
        "role" => role_detail(&fetch_ns!(Role, client, namespace, name)),
        "clusterrole" => cluster_role_detail(&fetch_all!(ClusterRole, client, name)),
        "rolebinding" => role_binding_detail(&fetch_ns!(RoleBinding, client, namespace, name)),
        "clusterrolebinding" => {
            cluster_role_binding_detail(&fetch_all!(ClusterRoleBinding, client, name))
        }
        "storageclass" => storage_class_detail(&fetch_all!(StorageClass, client, name)),
        "endpointslice" => {
            endpoint_slice_detail(&fetch_ns!(EndpointSlice, client, namespace, name))
        }
        "ingressclass" => ingress_class_detail(&fetch_all!(IngressClass, client, name)),
        "pdb" => pdb_detail(&fetch_ns!(PodDisruptionBudget, client, namespace, name)),
        "resourcequota" => {
            resource_quota_detail(&fetch_ns!(ResourceQuota, client, namespace, name))
        }
        "limitrange" => limit_range_detail(&fetch_ns!(LimitRange, client, namespace, name)),
        "priorityclass" => priority_class_detail(&fetch_all!(PriorityClass, client, name)),
        "crd" => crd_detail(&fetch_all!(CustomResourceDefinition, client, name)),
        _ => return Err(unknown(kind)),
    })
}

pub async fn resource_yaml(
    client: &Client,
    kind: &str,
    namespace: Option<&str>,
    name: &str,
) -> Result<String> {
    Ok(match kind {
        "role" => serde_yaml::to_string(&fetch_ns!(Role, client, namespace, name))?,
        "clusterrole" => serde_yaml::to_string(&fetch_all!(ClusterRole, client, name))?,
        "rolebinding" => serde_yaml::to_string(&fetch_ns!(RoleBinding, client, namespace, name))?,
        "clusterrolebinding" => {
            serde_yaml::to_string(&fetch_all!(ClusterRoleBinding, client, name))?
        }
        "storageclass" => serde_yaml::to_string(&fetch_all!(StorageClass, client, name))?,
        "endpointslice" => {
            serde_yaml::to_string(&fetch_ns!(EndpointSlice, client, namespace, name))?
        }
        "ingressclass" => serde_yaml::to_string(&fetch_all!(IngressClass, client, name))?,
        "pdb" => serde_yaml::to_string(&fetch_ns!(PodDisruptionBudget, client, namespace, name))?,
        "resourcequota" => {
            serde_yaml::to_string(&fetch_ns!(ResourceQuota, client, namespace, name))?
        }
        "limitrange" => serde_yaml::to_string(&fetch_ns!(LimitRange, client, namespace, name))?,
        "priorityclass" => serde_yaml::to_string(&fetch_all!(PriorityClass, client, name))?,
        "crd" => serde_yaml::to_string(&fetch_all!(CustomResourceDefinition, client, name))?,
        _ => return Err(unknown(kind)),
    })
}

/// Events whose involvedObject is this object. Cluster-scoped objects get
/// their events in the `default` namespace or none at all, so those search
/// every namespace.
pub async fn object_events(
    client: &Client,
    api_kind: &str,
    namespace: Option<&str>,
    name: &str,
) -> Result<Vec<GenericEvent>> {
    let api: Api<Event> = match namespace.filter(|n| !n.is_empty()) {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let lp = ListParams::default().fields(&format!(
        "involvedObject.name={name},involvedObject.kind={api_kind}"
    ));
    let list = api.list(&lp).await?;
    Ok(list
        .items
        .iter()
        .map(|e| GenericEvent {
            event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
            reason: e.reason.clone().unwrap_or_default(),
            message: e.message.clone().unwrap_or_default(),
            count: e.count.unwrap_or(1),
            first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            source: e
                .source
                .as_ref()
                .and_then(|s| s.component.clone())
                .unwrap_or_default(),
        })
        .collect())
}

pub async fn resource_events(
    client: &Client,
    kind: &str,
    namespace: Option<&str>,
    name: &str,
) -> Result<Vec<GenericEvent>> {
    let (api_kind, namespaced) = kind_info(kind).ok_or_else(|| unknown(kind))?;
    object_events(
        client,
        api_kind,
        if namespaced { namespace } else { None },
        name,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::discovery::v1::{Endpoint, EndpointConditions};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelectorRequirement;

    fn subject(kind: &str, name: &str, ns: Option<&str>) -> Subject {
        Subject {
            kind: kind.into(),
            name: name.into(),
            namespace: ns.map(Into::into),
            api_group: None,
        }
    }

    #[test]
    fn subjects_summary_shows_two_then_a_count() {
        assert_eq!(subjects_summary(&None), "");
        let s = Some(vec![
            subject("User", "alice", None),
            subject("ServiceAccount", "ci", Some("build")),
            subject("Group", "devs", None),
            subject("Group", "ops", None),
        ]);
        assert_eq!(
            subjects_summary(&s),
            "User alice, ServiceAccount build/ci +2"
        );
    }

    #[test]
    fn selector_string_covers_labels_and_expressions() {
        assert_eq!(selector_string(&None), "");
        assert_eq!(selector_string(&Some(LabelSelector::default())), "<all>");
        let sel = LabelSelector {
            match_labels: Some(BTreeMap::from([("app".to_string(), "web".to_string())])),
            match_expressions: Some(vec![
                LabelSelectorRequirement {
                    key: "tier".into(),
                    operator: "In".into(),
                    values: Some(vec!["api".into(), "db".into()]),
                },
                LabelSelectorRequirement {
                    key: "canary".into(),
                    operator: "DoesNotExist".into(),
                    values: None,
                },
            ]),
        };
        assert_eq!(
            selector_string(&Some(sel)),
            "app=web, tier in (api,db), !canary"
        );
    }

    #[test]
    fn rule_rows_name_the_core_group_and_fall_back_to_urls() {
        let rules = Some(vec![
            PolicyRule {
                api_groups: Some(vec!["".into(), "apps".into()]),
                resources: Some(vec!["pods".into(), "deployments".into()]),
                verbs: vec!["get".into(), "list".into()],
                ..Default::default()
            },
            PolicyRule {
                non_resource_urls: Some(vec!["/healthz".into()]),
                verbs: vec!["get".into()],
                ..Default::default()
            },
        ]);
        let rows = rule_rows(&rules);
        assert_eq!(
            rows[0],
            vec!["core, apps", "pods, deployments", "get, list", ""]
        );
        assert_eq!(rows[1][1], "/healthz");
    }

    #[test]
    fn slice_ready_treats_missing_condition_as_ready() {
        let ep = |ready: Option<bool>| Endpoint {
            addresses: vec!["10.0.0.1".into()],
            conditions: Some(EndpointConditions {
                ready,
                ..Default::default()
            }),
            ..Default::default()
        };
        let slice = EndpointSlice {
            address_type: "IPv4".into(),
            endpoints: Some(vec![ep(Some(true)), ep(None), ep(Some(false))]),
            ..Default::default()
        };
        assert_eq!(slice_ready(&slice), (2, 3));
    }

    #[test]
    fn empty_fields_render_as_a_dash_and_empty_tables_are_dropped() {
        let d = base("Role", &ObjectMeta::default())
            .field("Rules", "")
            .table("Rules", &["A"], vec![]);
        assert_eq!(d.fields[0].value, "–");
        assert!(d.tables.is_empty());
    }

    #[test]
    fn custom_target_builds_api_version() {
        let t = CustomTarget {
            group: "cert-manager.io".into(),
            version: "v1".into(),
            kind: "Certificate".into(),
            plural: "certificates".into(),
            namespaced: true,
        };
        assert_eq!(t.api_resource().api_version, "cert-manager.io/v1");
        let core = CustomTarget {
            group: String::new(),
            ..t
        };
        assert_eq!(core.api_resource().api_version, "v1");
    }

    #[test]
    fn kind_info_knows_scope() {
        assert_eq!(kind_info("pdb"), Some(("PodDisruptionBudget", true)));
        assert_eq!(kind_info("clusterrole"), Some(("ClusterRole", false)));
        assert_eq!(kind_info("pod"), None);
    }
}
