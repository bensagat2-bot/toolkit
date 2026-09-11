export type OpFieldType = 'text' | 'file' | 'select'

export interface OpField {
  key: 'part' | 'file' | 'offset' | 'size' | 'mode' | 'value'
  label: string
  type: OpFieldType
  placeholder?: string
  options?: string[]
}

export interface UnisocOpDef {
  name: string
  label: string
  group: string
  danger?: boolean
  fields?: OpField[]
}

export const OPS_GROUPS: Array<{ id: string; label: string }> = [
  { id: 'flash', label: 'Flash / Format' },
  { id: 'backup', label: 'Backup / Restore' },
  { id: 'partition', label: 'Partition Table' },
  { id: 'boot', label: 'Boot / Exit Mode' },
  { id: 'security', label: 'Security / Service' },
]

export const UNISOC_OPS: UnisocOpDef[] = [
  // Flash / Format
  { name: 'erase_all', label: 'Format All', group: 'flash', danger: true },
  { name: 'erase', label: 'Erase Partition', group: 'flash', danger: true, fields: [{ key: 'part', label: 'Partition', type: 'text' }] },
  { name: 'write', label: 'Flash Partition', group: 'flash', fields: [
    { key: 'part', label: 'Partition', type: 'text' },
    { key: 'file', label: 'Image', type: 'file' },
  ] },
  // Backup / Restore
  { name: 'read_lite', label: 'Lite Backup (all_lite)', group: 'backup' },
  { name: 'read', label: 'Backup Partition', group: 'backup', fields: [{ key: 'part', label: 'Partition', type: 'text' }] },
  { name: 'write', label: 'Restore Partition', group: 'backup', fields: [
    { key: 'part', label: 'Partition', type: 'text' },
    { key: 'file', label: 'Image', type: 'file' },
  ] },
  { name: 'read_part', label: 'Read at Offset', group: 'backup', fields: [
    { key: 'part', label: 'Partition', type: 'text' },
    { key: 'offset', label: 'Offset', type: 'text', placeholder: 'hex or dec' },
    { key: 'size', label: 'Size', type: 'text', placeholder: 'hex or dec' },
    { key: 'file', label: 'Output', type: 'file' },
  ] },
  { name: 'write_offset', label: 'Write at Offset', group: 'backup', fields: [
    { key: 'part', label: 'Partition', type: 'text' },
    { key: 'offset', label: 'Offset', type: 'text', placeholder: 'hex or dec' },
    { key: 'file', label: 'Image', type: 'file' },
  ] },
  // Partition table
  { name: 'list', label: 'Print Partition List', group: 'partition' },
  { name: 'partition_list', label: 'Dump Partition Table', group: 'partition', fields: [{ key: 'file', label: 'Output', type: 'file' }] },
  { name: 'size_part', label: 'Partition Size', group: 'partition', fields: [{ key: 'part', label: 'Partition', type: 'text' }] },
  { name: 'check_part', label: 'Check Partition', group: 'partition', fields: [{ key: 'part', label: 'Partition', type: 'text' }] },
  { name: 'repartition', label: 'Repartition from XML', group: 'partition', danger: true, fields: [{ key: 'file', label: 'XML', type: 'file' }] },
  { name: 'set_active', label: 'Set Active Slot', group: 'partition', fields: [{ key: 'mode', label: 'Slot', type: 'select', options: ['a', 'b'] }] },
  { name: 'verity', label: 'Toggle dm-verity', group: 'partition', fields: [{ key: 'value', label: 'Value', type: 'select', options: ['0', '1'] }] },
  // Boot / exit mode
  { name: 'reboot_recovery', label: 'Reboot Recovery', group: 'boot' },
  { name: 'reboot_fastboot', label: 'Reboot Fastboot', group: 'boot' },
  { name: 'poweroff', label: 'Power Off', group: 'boot' },
  { name: 'firstmode', label: 'Set Boot Mode', group: 'boot', fields: [{ key: 'mode', label: 'Mode', type: 'select', options: ['normal', 'recovery', 'fastboot'] }] },
  // Security / service
  { name: 'misc_fix', label: 'Write Misc Red-State Fix', group: 'security', fields: [{ key: 'file', label: 'misc bin (optional)', type: 'file' }] },
  { name: 'backup_nv', label: 'Backup NV (prodnv/nvdata/nvcfg)', group: 'security' },
  { name: 'restore_nv', label: 'Restore NV from Files', group: 'security', fields: [
    { key: 'file', label: 'prodnv file', type: 'file' },
    { key: 'value', label: 'nvdata file', type: 'file' },
  ] },
]