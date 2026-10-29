<script lang="ts">
  import { guard } from '../../stores/detail';
  import { invoke } from '@tauri-apps/api/core';
  import { currentContext, selectedNamespace, namespaces, loadNetworkPolicies } from '../../stores/kubernetes';
  import { untrack } from 'svelte';
  import CustomSelect from './CustomSelect.svelte';
  import { modal, escapeKeydown } from '../../actions/modal';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import { toast } from '../../stores/toast';

  // Edit data structure matching backend NetworkPolicyDetail
  interface EditData {
    name: string;
    namespace: string;
    pod_selector: Record<string, string>;
    policy_types: string[];
    ingress_rules: {
      from: { peer_type: string; pod_selector: Record<string, string>; namespace_selector: Record<string, string>; ip_block: string | null }[];
      ports: { protocol: string; port: string; end_port: number | null }[];
    }[];
    egress_rules: {
      to: { peer_type: string; pod_selector: Record<string, string>; namespace_selector: Record<string, string>; ip_block: string | null }[];
      ports: { protocol: string; port: string; end_port: number | null }[];
    }[];
  }

  interface Props {
    show: boolean;
    onclose: () => void;
    editData?: EditData | null;
    /**
     * The context this form targets, fixed by the caller when it opens (for
     * an edit: the context the row was loaded from). Every call uses it,
     * never the live context, and the editor closes if the live context
     * moves (a switch, or an external kubectx picked up by polling).
     */
    context: string;
  }

  let { show, onclose, editData = null, context }: Props = $props();

  $effect(() => {
    if (show && context && $currentContext !== context) {
      toast(`Context changed to ${$currentContext}. Closed the editor for ${context}.`);
      confirmDelete = false;
      handleClose();
    }
  });

  // Delete lives here, in edit mode, behind a ConfirmDialog: the list rows
  // never delete (CLAUDE.md, Architecture Note 7), and the editor is the one
  // place the user has already picked this exact object.
  let confirmDelete = $state(false);
  let deleted = false;

  async function deleteEdited() {
    if (!editData) return;
    await invoke('delete_network_policy', { contextName: context, namespace: editData.namespace, name: editData.name });
    deleted = true;
    loadNetworkPolicies($selectedNamespace);
  }

  // Form state
  let name = $state('');
  let editNamespace = $state<string | null>(null);
  let createNamespace = $state<string>('');
  let podSelectorLabels = $state<{key: string, value: string}[]>([]);
  let policyTypes = $state<string[]>(['Ingress']);

  // Namespace options for dropdown
  const namespaceOptions = $derived(
    $namespaces.map(ns => ({ value: ns, label: ns }))
  );

  // Rules
  let ingressRules = $state<IngressEgressRule[]>([]);
  let egressRules = $state<IngressEgressRule[]>([]);

  // UI state
  let isCreating = $state(false);
  let error = $state<string | null>(null);
  let activeTab = $state<'basic' | 'ingress' | 'egress'>('basic');

  // Track if we're in edit mode
  let isEditMode = $derived(editData !== null);

  interface Peer {
    peerType: 'pod' | 'namespace' | 'ipBlock';
    podSelector: {key: string, value: string}[];
    namespaceSelector: {key: string, value: string}[];
    ipBlock: string;
    ipBlockExcept: string[];
  }

  interface Port {
    protocol: string;
    port: number | null;
    endPort: number | null;
  }

  interface IngressEgressRule {
    peers: Peer[];
    ports: Port[];
  }

  // Convert backend label map to array format
  function mapToLabels(map: Record<string, string> | null | undefined): {key: string, value: string}[] {
    if (!map) return [];
    return Object.entries(map).map(([key, value]) => ({ key, value }));
  }

  // Convert backend peer to frontend format
  function convertBackendPeer(backendPeer: { peer_type: string; pod_selector: Record<string, string>; namespace_selector: Record<string, string>; ip_block: string | null }): Peer {
    let peerType: 'pod' | 'namespace' | 'ipBlock' = 'pod';
    if (backendPeer.peer_type === 'IPBlock') {
      peerType = 'ipBlock';
    } else if (backendPeer.peer_type === 'Namespace' || (Object.keys(backendPeer.namespace_selector || {}).length > 0 && Object.keys(backendPeer.pod_selector || {}).length === 0)) {
      peerType = 'namespace';
    }

    return {
      peerType,
      podSelector: mapToLabels(backendPeer.pod_selector),
      namespaceSelector: mapToLabels(backendPeer.namespace_selector),
      ipBlock: backendPeer.ip_block || '',
      ipBlockExcept: []
    };
  }

  // Convert backend port to frontend format
  function convertBackendPort(backendPort: { protocol: string; port: string; end_port: number | null }): Port {
    return {
      protocol: backendPort.protocol || 'TCP',
      port: backendPort.port ? parseInt(backendPort.port, 10) : null,
      endPort: backendPort.end_port
    };
  }

  // Populate form from edit data
  function populateFromEditData(data: EditData) {
    name = data.name;
    editNamespace = data.namespace;
    podSelectorLabels = mapToLabels(data.pod_selector);
    policyTypes = data.policy_types || ['Ingress'];

    // Convert ingress rules
    ingressRules = (data.ingress_rules || []).map(rule => ({
      peers: (rule.from || []).map(convertBackendPeer),
      ports: (rule.ports || []).map(convertBackendPort)
    }));

    // Convert egress rules
    egressRules = (data.egress_rules || []).map(rule => ({
      peers: (rule.to || []).map(convertBackendPeer),
      ports: (rule.ports || []).map(convertBackendPort)
    }));
  }

  // Watch for editData changes and populate form

  // Unsaved-changes check against the form as it was when it was filled.
  const formKey = $derived(JSON.stringify({ name, editNamespace, createNamespace, podSelectorLabels, policyTypes, ingressRules, egressRules }));
  let baseline = $state('');
  // Fill the form when the editor opens (or is pointed at another object),
  // then take the baseline from that. Runs on `show`/`editData` only, so a
  // namespace list reload never resets typed input or re-baselines a
  // dirty form as clean.
  $effect(() => {
    if (!show) return;
    const data = editData;
    untrack(() => {
      if (data) {
        populateFromEditData(data);
      } else {
        reset();
        createNamespace = $selectedNamespace || $namespaces[0] || 'default';
      }
      baseline = formKey;
    });
  });
  // One close rule for Escape, Cancel, × and the backdrop: unsaved edits
  // ask first, a clean form just closes.
  let confirmDiscard = $state(false);
  function requestClose() {
    if (formKey !== baseline) {
      confirmDiscard = true;
      return;
    }
    handleClose();
  }

  function reset() {
    name = '';
    editNamespace = null;
    createNamespace = $selectedNamespace || 'default';
    podSelectorLabels = [];
    policyTypes = ['Ingress'];
    ingressRules = [];
    egressRules = [];
    error = null;
    activeTab = 'basic';
  }

  function addPodSelectorLabel() {
    podSelectorLabels = [...podSelectorLabels, { key: '', value: '' }];
  }

  function removePodSelectorLabel(index: number) {
    podSelectorLabels = podSelectorLabels.filter((_, i) => i !== index);
  }

  function togglePolicyType(type: string) {
    if (policyTypes.includes(type)) {
      policyTypes = policyTypes.filter(t => t !== type);
    } else {
      policyTypes = [...policyTypes, type];
    }
  }

  function addIngressRule() {
    ingressRules = [...ingressRules, { peers: [], ports: [] }];
  }

  function removeIngressRule(index: number) {
    ingressRules = ingressRules.filter((_, i) => i !== index);
  }

  function addEgressRule() {
    egressRules = [...egressRules, { peers: [], ports: [] }];
  }

  function removeEgressRule(index: number) {
    egressRules = egressRules.filter((_, i) => i !== index);
  }

  function addPeer(rules: IngressEgressRule[], ruleIndex: number, isIngress: boolean) {
    const newPeer: Peer = {
      peerType: 'pod',
      podSelector: [],
      namespaceSelector: [],
      ipBlock: '',
      ipBlockExcept: []
    };
    rules[ruleIndex].peers = [...rules[ruleIndex].peers, newPeer];
    if (isIngress) {
      ingressRules = [...rules];
    } else {
      egressRules = [...rules];
    }
  }

  function removePeer(rules: IngressEgressRule[], ruleIndex: number, peerIndex: number, isIngress: boolean) {
    rules[ruleIndex].peers = rules[ruleIndex].peers.filter((_, i) => i !== peerIndex);
    if (isIngress) {
      ingressRules = [...rules];
    } else {
      egressRules = [...rules];
    }
  }

  function addPort(rules: IngressEgressRule[], ruleIndex: number, isIngress: boolean) {
    const newPort: Port = { protocol: 'TCP', port: null, endPort: null };
    rules[ruleIndex].ports = [...rules[ruleIndex].ports, newPort];
    if (isIngress) {
      ingressRules = [...rules];
    } else {
      egressRules = [...rules];
    }
  }

  function removePort(rules: IngressEgressRule[], ruleIndex: number, portIndex: number, isIngress: boolean) {
    rules[ruleIndex].ports = rules[ruleIndex].ports.filter((_, i) => i !== portIndex);
    if (isIngress) {
      ingressRules = [...rules];
    } else {
      egressRules = [...rules];
    }
  }

  function addPeerLabel(peer: Peer, type: 'pod' | 'namespace') {
    if (type === 'pod') {
      peer.podSelector = [...peer.podSelector, { key: '', value: '' }];
    } else {
      peer.namespaceSelector = [...peer.namespaceSelector, { key: '', value: '' }];
    }
  }

  function removePeerLabel(peer: Peer, type: 'pod' | 'namespace', index: number) {
    if (type === 'pod') {
      peer.podSelector = peer.podSelector.filter((_, i) => i !== index);
    } else {
      peer.namespaceSelector = peer.namespaceSelector.filter((_, i) => i !== index);
    }
  }

  function labelsToMap(labels: {key: string, value: string}[]): Record<string, string> {
    const map: Record<string, string> = {};
    for (const label of labels) {
      if (label.key.trim()) {
        map[label.key.trim()] = label.value.trim();
      }
    }
    return map;
  }

  async function handleSave() {
    if (!guard('edit')) return;
    if (!name.trim()) {
      error = 'Name is required';
      return;
    }
    if (policyTypes.length === 0) {
      error = 'At least one policy type is required';
      return;
    }

    // Use edit namespace if in edit mode, otherwise use createNamespace
    const namespace = isEditMode ? (editNamespace || 'default') : (createNamespace || 'default');

    isCreating = true;
    error = null;

    try {
      // Convert to backend format
      const podSelector = labelsToMap(podSelectorLabels);

      const convertedIngressRules = ingressRules.map(rule => ({
        peers: rule.peers.map(peer => ({
          peer_type: peer.peerType,
          pod_selector: labelsToMap(peer.podSelector),
          namespace_selector: labelsToMap(peer.namespaceSelector),
          ip_block: peer.peerType === 'ipBlock' ? peer.ipBlock : null,
          ip_block_except: peer.ipBlockExcept.filter(e => e.trim()),
        })),
        ports: rule.ports.map(port => ({
          protocol: port.protocol,
          port: port.port,
          end_port: port.endPort,
        })),
      }));

      const convertedEgressRules = egressRules.map(rule => ({
        peers: rule.peers.map(peer => ({
          peer_type: peer.peerType,
          pod_selector: labelsToMap(peer.podSelector),
          namespace_selector: labelsToMap(peer.namespaceSelector),
          ip_block: peer.peerType === 'ipBlock' ? peer.ipBlock : null,
          ip_block_except: peer.ipBlockExcept.filter(e => e.trim()),
        })),
        ports: rule.ports.map(port => ({
          protocol: port.protocol,
          port: port.port,
          end_port: port.endPort,
        })),
      }));

      if (isEditMode) {
        await invoke('update_network_policy', {
          contextName: context,
          namespace,
          name: name.trim(),
          podSelector,
          policyTypes,
          ingressRules: convertedIngressRules,
          egressRules: convertedEgressRules,
        });
      } else {
        await invoke('create_network_policy', {
          contextName: context,
          namespace,
          name: name.trim(),
          podSelector,
          policyTypes,
          ingressRules: convertedIngressRules,
          egressRules: convertedEgressRules,
        });
      }

      await loadNetworkPolicies(namespace);
      reset();
      onclose();
    } catch (e) {
      error = String(e);
    } finally {
      isCreating = false;
    }
  }

  function handleClose() {
    confirmDiscard = false;
    reset();
    onclose();
  }
</script>

{#if show}
  <div role="presentation" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onmousedown={(e) => e.target === e.currentTarget && requestClose()}>
    <div role="dialog" aria-modal="true" aria-labelledby="np-title" tabindex="-1" use:modal onkeydown={escapeKeydown(requestClose)}
      class="bg-bg-secondary rounded-lg border border-border-subtle w-[800px] max-w-[calc(100vw-16px)] max-h-[85vh] flex flex-col"
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-border-subtle">
        <h2 id="np-title" class="text-[18px]/7 font-semibold text-text-primary">{isEditMode ? 'Edit' : 'Create'} Network Policy</h2>
        <button aria-label="Close"
          onclick={requestClose}
          class="p-1 hover:bg-bg-tertiary rounded-sm transition-colors"
        >
          <svg class="w-5 h-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-border-subtle">
        <button
          class="px-4 py-2 text-base font-medium transition-colors {activeTab === 'basic' ? 'text-accent-text border-b-2 border-accent-primary' : 'text-text-secondary hover:text-text-primary'}"
          onclick={() => activeTab = 'basic'}
        >
          Basic
        </button>
        <button
          class="px-4 py-2 text-base font-medium transition-colors {activeTab === 'ingress' ? 'text-accent-text border-b-2 border-accent-primary' : 'text-text-secondary hover:text-text-primary'}"
          onclick={() => activeTab = 'ingress'}
        >
          Ingress Rules ({ingressRules.length})
        </button>
        <button
          class="px-4 py-2 text-base font-medium transition-colors {activeTab === 'egress' ? 'text-accent-text border-b-2 border-accent-primary' : 'text-text-secondary hover:text-text-primary'}"
          onclick={() => activeTab = 'egress'}
        >
          Egress Rules ({egressRules.length})
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if activeTab === 'basic'}
          <!-- Basic Settings -->
          <div class="space-y-4">
            <!-- Name -->
            <div>
              <label for="np-name" class="block text-base font-medium text-text-secondary mb-1">Name *</label>
              {#if isEditMode}
                <div class="text-base text-text-primary bg-bg-tertiary border border-border-subtle rounded-sm px-3 py-2 opacity-60">
                  {name}
                </div>
                <p class="text-xs text-text-muted mt-1">Name cannot be changed when editing</p>
              {:else}
                <input id="np-name"
                  type="text"
                  bind:value={name}
                  placeholder="my-network-policy"
                  class="w-full bg-bg-tertiary border border-border-subtle rounded-sm px-3 py-2 text-text-primary text-base focus:outline-hidden focus:border-accent-primary"
                />
              {/if}
            </div>

            <!-- Namespace -->
            <div>
              <span class="block text-base font-medium text-text-secondary mb-1">Namespace *</span>
              {#if isEditMode}
                <div class="text-base text-text-primary bg-bg-tertiary border border-border-subtle rounded-sm px-3 py-2 opacity-60">
                  {editNamespace || 'default'}
                </div>
                <p class="text-xs text-text-muted mt-1">Namespace cannot be changed when editing</p>
              {:else}
                <CustomSelect
                  ariaLabel="Namespace"
                  value={createNamespace}
                  options={namespaceOptions}
                  placeholder="Select namespace..."
                  onchange={(v) => createNamespace = v}
                />
              {/if}
            </div>

            <!-- Policy Types -->
            <div>
              <span id="np-policy-types" class="block text-base font-medium text-text-secondary mb-2">Policy Types *</span>
              <div class="flex gap-4" role="group" aria-labelledby="np-policy-types">
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={policyTypes.includes('Ingress')}
                    onchange={() => togglePolicyType('Ingress')}
                    class="w-4 h-4 rounded-sm bg-bg-tertiary border-border-subtle text-accent-text focus:ring-accent-primary"
                  />
                  <span class="text-base text-text-primary">Ingress</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={policyTypes.includes('Egress')}
                    onchange={() => togglePolicyType('Egress')}
                    class="w-4 h-4 rounded-sm bg-bg-tertiary border-border-subtle text-accent-text focus:ring-accent-primary"
                  />
                  <span class="text-base text-text-primary">Egress</span>
                </label>
              </div>
              <p class="text-xs text-text-muted mt-1">Selected types without rules will deny all traffic of that type</p>
            </div>

            <!-- Pod Selector -->
            <div>
              <div class="flex items-center justify-between mb-2">
                <span class="block text-base font-medium text-text-secondary">Pod Selector</span>
                <button
                  onclick={addPodSelectorLabel}
                  class="text-xs px-2 py-1 bg-accent-primary/10 text-accent-text rounded-sm hover:bg-accent-primary/20 transition-colors"
                >
                  + Add Label
                </button>
              </div>
              {#if podSelectorLabels.length === 0}
                <p class="text-xs text-text-muted">No labels = applies to all pods in namespace</p>
              {:else}
                <div class="space-y-2">
                  {#each podSelectorLabels as label, i}
                    <div class="flex gap-2 items-center">
                      <input
                        type="text"
                        bind:value={label.key}
                        placeholder="key"
                        class="flex-1 bg-bg-tertiary border border-border-subtle rounded-sm px-3 py-1.5 text-text-primary text-base focus:outline-hidden focus:border-accent-primary"
                      />
                      <span class="text-text-muted">=</span>
                      <input
                        type="text"
                        bind:value={label.value}
                        placeholder="value"
                        class="flex-1 bg-bg-tertiary border border-border-subtle rounded-sm px-3 py-1.5 text-text-primary text-base focus:outline-hidden focus:border-accent-primary"
                      />
                      <button aria-label={`Remove selector label ${i + 1}`}
                        onclick={() => removePodSelectorLabel(i)}
                        class="p-1 text-danger hover:bg-accent-error/10 rounded-sm"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          </div>

        {:else if activeTab === 'ingress'}
          <!-- Ingress Rules -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <p class="text-base text-text-muted">
                {#if !policyTypes.includes('Ingress')}
                  <span class="text-warning">Warning: Ingress not selected in policy types</span>
                {:else if ingressRules.length === 0}
                  No ingress rules = deny all incoming traffic
                {:else}
                  {ingressRules.length} rule(s) defined
                {/if}
              </p>
              <button
                onclick={addIngressRule}
                class="text-base px-3 py-1.5 bg-btn text-on-btn rounded-sm hover:opacity-90 transition-colors"
              >
                + Add Ingress Rule
              </button>
            </div>

            {#each ingressRules as rule, ruleIndex}
              <div class="bg-bg-tertiary rounded-lg p-4 border border-border-subtle">
                <div class="flex items-center justify-between mb-3">
                  <span class="text-base font-medium text-success">Ingress Rule {ruleIndex + 1}</span>
                  <button
                    onclick={() => removeIngressRule(ruleIndex)}
                    class="text-xs px-2 py-1 text-danger hover:bg-accent-error/10 rounded-sm"
                  >
                    Remove Rule
                  </button>
                </div>

                <!-- From (Peers) -->
                <div class="mb-4">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs text-text-muted uppercase">From (Sources)</span>
                    <button
                      onclick={() => addPeer(ingressRules, ruleIndex, true)}
                      class="text-xs px-2 py-1 bg-bg-secondary text-text-secondary rounded-sm hover:text-text-primary"
                    >
                      + Add Source
                    </button>
                  </div>
                  {#if rule.peers.length === 0}
                    <p class="text-xs text-text-muted">No sources = allow from anywhere</p>
                  {:else}
                    <div class="space-y-3">
                      {#each rule.peers as peer, peerIndex}
                        <div class="bg-bg-secondary rounded-sm p-3">
                          <div class="flex items-center justify-between mb-2">
                            <CustomSelect
                              ariaLabel="Peer type"
                              value={peer.peerType}
                              options={[
                                { value: 'pod', label: 'Pod Selector' },
                                { value: 'namespace', label: 'Namespace Selector' },
                                { value: 'ipBlock', label: 'IP Block (CIDR)' }
                              ]}
                              onchange={(v) => peer.peerType = v as 'pod' | 'namespace' | 'ipBlock'}
                            />
                            <button aria-label={`Remove peer ${peerIndex + 1}`}
                              onclick={() => removePeer(ingressRules, ruleIndex, peerIndex, true)}
                              class="p-1 text-danger hover:bg-accent-error/10 rounded-sm"
                            >
                              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                              </svg>
                            </button>
                          </div>

                          {#if peer.peerType === 'ipBlock'}
                            <input
                              type="text"
                              bind:value={peer.ipBlock}
                              placeholder="10.0.0.0/8"
                              class="w-full bg-bg-tertiary border border-border-subtle rounded-sm px-3 py-1.5 text-text-primary text-base"
                            />
                          {:else}
                            <div class="space-y-2">
                              {#if peer.peerType === 'pod' || peer.peerType === 'namespace'}
                                <div>
                                  <div class="flex items-center justify-between mb-1">
                                    <span class="text-xs text-text-muted">
                                      {peer.peerType === 'pod' ? 'Pod Labels' : 'Namespace Labels'}
                                    </span>
                                    <button
                                      onclick={() => addPeerLabel(peer, peer.peerType === 'pod' ? 'pod' : 'namespace')}
                                      class="text-xs text-accent-text"
                                    >
                                      + Label
                                    </button>
                                  </div>
                                  {#each peer.peerType === 'pod' ? peer.podSelector : peer.namespaceSelector as label, li}
                                    <div class="flex gap-2 items-center">
                                      <input
                                        type="text"
                                        bind:value={label.key}
                                        placeholder="key"
                                        class="flex-1 bg-bg-tertiary border border-border-subtle rounded-sm px-2 py-1 text-text-primary text-xs"
                                      />
                                      <span class="text-text-muted text-xs">=</span>
                                      <input
                                        type="text"
                                        bind:value={label.value}
                                        placeholder="value"
                                        class="flex-1 bg-bg-tertiary border border-border-subtle rounded-sm px-2 py-1 text-text-primary text-xs"
                                      />
                                      <button aria-label={`Remove label ${li + 1}`}
                                        onclick={() => removePeerLabel(peer, peer.peerType === 'pod' ? 'pod' : 'namespace', li)}
                                        class="text-danger"
                                      >
                                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                        </svg>
                                      </button>
                                    </div>
                                  {/each}
                                  {#if (peer.peerType === 'pod' ? peer.podSelector : peer.namespaceSelector).length === 0}
                                    <p class="text-xs text-text-muted">Empty = match all</p>
                                  {/if}
                                </div>
                              {/if}
                            </div>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>

                <!-- Ports -->
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs text-text-muted uppercase">Ports</span>
                    <button
                      onclick={() => addPort(ingressRules, ruleIndex, true)}
                      class="text-xs px-2 py-1 bg-bg-secondary text-text-secondary rounded-sm hover:text-text-primary"
                    >
                      + Add Port
                    </button>
                  </div>
                  {#if rule.ports.length === 0}
                    <p class="text-xs text-text-muted">No ports = allow all ports</p>
                  {:else}
                    <div class="flex flex-wrap gap-2">
                      {#each rule.ports as port, portIndex}
                        <div class="flex items-center gap-2 bg-bg-secondary rounded-sm px-2 py-1">
                          <select
                            bind:value={port.protocol}
                            class="bg-transparent text-xs text-text-primary focus:outline-hidden"
                          >
                            <option value="TCP">TCP</option>
                            <option value="UDP">UDP</option>
                            <option value="SCTP">SCTP</option>
                          </select>
                          <input
                            type="number"
                            bind:value={port.port}
                            placeholder="port"
                            class="w-16 bg-bg-tertiary border border-border-subtle rounded-sm px-2 py-0.5 text-text-primary text-xs"
                          />
                          <button aria-label={`Remove port ${portIndex + 1}`}
                            onclick={() => removePort(ingressRules, ruleIndex, portIndex, true)}
                            class="text-danger"
                          >
                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                          </button>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>

        {:else if activeTab === 'egress'}
          <!-- Egress Rules -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <p class="text-base text-text-muted">
                {#if !policyTypes.includes('Egress')}
                  <span class="text-warning">Warning: Egress not selected in policy types</span>
                {:else if egressRules.length === 0}
                  No egress rules = deny all outgoing traffic
                {:else}
                  {egressRules.length} rule(s) defined
                {/if}
              </p>
              <button
                onclick={addEgressRule}
                class="text-base px-3 py-1.5 bg-btn text-on-btn rounded-sm hover:opacity-90 transition-colors"
              >
                + Add Egress Rule
              </button>
            </div>

            {#each egressRules as rule, ruleIndex}
              <div class="bg-bg-tertiary rounded-lg p-4 border border-border-subtle">
                <div class="flex items-center justify-between mb-3">
                  <span class="text-base font-medium text-warning">Egress Rule {ruleIndex + 1}</span>
                  <button
                    onclick={() => removeEgressRule(ruleIndex)}
                    class="text-xs px-2 py-1 text-danger hover:bg-accent-error/10 rounded-sm"
                  >
                    Remove Rule
                  </button>
                </div>

                <!-- To (Peers) -->
                <div class="mb-4">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs text-text-muted uppercase">To (Destinations)</span>
                    <button
                      onclick={() => addPeer(egressRules, ruleIndex, false)}
                      class="text-xs px-2 py-1 bg-bg-secondary text-text-secondary rounded-sm hover:text-text-primary"
                    >
                      + Add Destination
                    </button>
                  </div>
                  {#if rule.peers.length === 0}
                    <p class="text-xs text-text-muted">No destinations = allow to anywhere</p>
                  {:else}
                    <div class="space-y-3">
                      {#each rule.peers as peer, peerIndex}
                        <div class="bg-bg-secondary rounded-sm p-3">
                          <div class="flex items-center justify-between mb-2">
                            <CustomSelect
                              ariaLabel="Peer type"
                              value={peer.peerType}
                              options={[
                                { value: 'pod', label: 'Pod Selector' },
                                { value: 'namespace', label: 'Namespace Selector' },
                                { value: 'ipBlock', label: 'IP Block (CIDR)' }
                              ]}
                              onchange={(v) => peer.peerType = v as 'pod' | 'namespace' | 'ipBlock'}
                            />
                            <button aria-label={`Remove peer ${peerIndex + 1}`}
                              onclick={() => removePeer(egressRules, ruleIndex, peerIndex, false)}
                              class="p-1 text-danger hover:bg-accent-error/10 rounded-sm"
                            >
                              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                              </svg>
                            </button>
                          </div>

                          {#if peer.peerType === 'ipBlock'}
                            <input
                              type="text"
                              bind:value={peer.ipBlock}
                              placeholder="10.0.0.0/8"
                              class="w-full bg-bg-tertiary border border-border-subtle rounded-sm px-3 py-1.5 text-text-primary text-base"
                            />
                          {:else}
                            <div class="space-y-2">
                              {#if peer.peerType === 'pod' || peer.peerType === 'namespace'}
                                <div>
                                  <div class="flex items-center justify-between mb-1">
                                    <span class="text-xs text-text-muted">
                                      {peer.peerType === 'pod' ? 'Pod Labels' : 'Namespace Labels'}
                                    </span>
                                    <button
                                      onclick={() => addPeerLabel(peer, peer.peerType === 'pod' ? 'pod' : 'namespace')}
                                      class="text-xs text-accent-text"
                                    >
                                      + Label
                                    </button>
                                  </div>
                                  {#each peer.peerType === 'pod' ? peer.podSelector : peer.namespaceSelector as label, li}
                                    <div class="flex gap-2 items-center">
                                      <input
                                        type="text"
                                        bind:value={label.key}
                                        placeholder="key"
                                        class="flex-1 bg-bg-tertiary border border-border-subtle rounded-sm px-2 py-1 text-text-primary text-xs"
                                      />
                                      <span class="text-text-muted text-xs">=</span>
                                      <input
                                        type="text"
                                        bind:value={label.value}
                                        placeholder="value"
                                        class="flex-1 bg-bg-tertiary border border-border-subtle rounded-sm px-2 py-1 text-text-primary text-xs"
                                      />
                                      <button aria-label={`Remove label ${li + 1}`}
                                        onclick={() => removePeerLabel(peer, peer.peerType === 'pod' ? 'pod' : 'namespace', li)}
                                        class="text-danger"
                                      >
                                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                        </svg>
                                      </button>
                                    </div>
                                  {/each}
                                  {#if (peer.peerType === 'pod' ? peer.podSelector : peer.namespaceSelector).length === 0}
                                    <p class="text-xs text-text-muted">Empty = match all</p>
                                  {/if}
                                </div>
                              {/if}
                            </div>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>

                <!-- Ports -->
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs text-text-muted uppercase">Ports</span>
                    <button
                      onclick={() => addPort(egressRules, ruleIndex, false)}
                      class="text-xs px-2 py-1 bg-bg-secondary text-text-secondary rounded-sm hover:text-text-primary"
                    >
                      + Add Port
                    </button>
                  </div>
                  {#if rule.ports.length === 0}
                    <p class="text-xs text-text-muted">No ports = allow all ports</p>
                  {:else}
                    <div class="flex flex-wrap gap-2">
                      {#each rule.ports as port, portIndex}
                        <div class="flex items-center gap-2 bg-bg-secondary rounded-sm px-2 py-1">
                          <select
                            bind:value={port.protocol}
                            class="bg-transparent text-xs text-text-primary focus:outline-hidden"
                          >
                            <option value="TCP">TCP</option>
                            <option value="UDP">UDP</option>
                            <option value="SCTP">SCTP</option>
                          </select>
                          <input
                            type="number"
                            bind:value={port.port}
                            placeholder="port"
                            class="w-16 bg-bg-tertiary border border-border-subtle rounded-sm px-2 py-0.5 text-text-primary text-xs"
                          />
                          <button aria-label={`Remove port ${portIndex + 1}`}
                            onclick={() => removePort(egressRules, ruleIndex, portIndex, false)}
                            class="text-danger"
                          >
                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                          </button>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between p-4 border-t border-border-subtle">
        {#if error}
          <p class="text-base text-danger">{error}</p>
        {:else if isEditMode}
          <button
            onclick={() => (confirmDelete = true)}
            disabled={isCreating}
            class="px-3 py-2 text-base text-danger rounded-sm hover:bg-hover transition-colors disabled:opacity-50"
          >
            Delete…
          </button>
        {:else}
          <div></div>
        {/if}
        <div class="flex gap-3">
          <button
            onclick={requestClose}
            class="px-4 py-2 text-base text-text-secondary hover:text-text-primary transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={handleSave}
            disabled={isCreating}
            class="px-4 py-2 text-base bg-btn text-on-btn rounded-sm hover:opacity-90 transition-colors disabled:opacity-50"
          >
            {isCreating ? (isEditMode ? 'Updating...' : 'Creating...') : (isEditMode ? 'Update Policy' : 'Create Policy')}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if confirmDelete && editData}
  <ConfirmDialog
    open={true}
    title={`Delete network policy ${editData.name}?`}
    resource={{ kind: 'NetworkPolicy', name: editData.name, namespace: editData.namespace, context }}
    body="Traffic this policy allows or blocks changes immediately for the pods it selects. This can't be undone."
    confirmLabel="Delete"
    tone="danger"
    successMessage={`Deleted ${editData.name}`}
    onConfirm={deleteEdited}
    onClose={() => {
      confirmDelete = false;
      if (deleted) {
        deleted = false;
        handleClose();
      }
    }}
  />
{/if}

{#if show && confirmDiscard}
  <ConfirmDialog
    open={true}
    title="Discard changes?"
    resource={{ kind: 'NetworkPolicy', name: name || 'new network policy', namespace: editNamespace ?? createNamespace, context }}
    body="Your edits to this network policy haven't been saved."
    confirmLabel="Discard"
    tone="danger"
    tier={2}
    prodEscalation={false}
    onConfirm={() => {
      confirmDiscard = false;
      handleClose();
    }}
    onClose={() => (confirmDiscard = false)}
  />
{/if}
