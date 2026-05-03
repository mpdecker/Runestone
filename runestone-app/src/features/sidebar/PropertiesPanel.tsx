import { useState } from 'react'
import { useStore } from '@/store'
import { Button } from '@/components/ui/button'

export function PropertiesPanel() {
  const {
    selectedNodeId,
    nodeProperties,
    setProperty,
    removeProperty,
  } = useStore()

  const [newKey, setNewKey] = useState('')
  const [newType, setNewType] = useState('text')
  const [newValue, setNewValue] = useState('')
  const [editKey, setEditKey] = useState<string | null>(null)
  const [editValue, setEditValue] = useState('')

  if (!selectedNodeId) return null

  const handleAdd = async () => {
    if (!newKey.trim()) return
    let value: unknown = newValue
    if (newType === 'number') value = Number(newValue)
    if (newType === 'checkbox') value = newValue.toLowerCase() === 'true'

    await setProperty(selectedNodeId, newKey.trim(), value)
    setNewKey('')
    setNewValue('')
  }

  const handleEdit = async (key: string, propType: string) => {
    if (editValue === '') return
    let value: unknown = editValue
    if (propType === 'number') value = Number(editValue)
    if (propType === 'checkbox') value = editValue.toLowerCase() === 'true'

    await setProperty(selectedNodeId, key, value)
    setEditKey(null)
  }

  const startEdit = (key: string, currentValue: unknown) => {
    setEditKey(key)
    setEditValue(String(currentValue ?? ''))
  }

  return (
    <div className="border-t p-2 space-y-1">
      <p className="text-[10px] text-muted-foreground uppercase tracking-wider">Properties</p>

      {nodeProperties.length > 0 ? (
        <div className="space-y-0.5">
          {nodeProperties.map((prop) => (
            <div key={prop.key} className="flex items-center gap-1 group">
              <span className="text-[10px] text-muted-foreground shrink-0">{prop.key}:</span>
              {editKey === prop.key ? (
                <div className="flex items-center gap-1 flex-1 min-w-0">
                  <input
                    className="flex-1 px-1 py-0.5 text-[10px] border rounded bg-background min-w-0"
                    value={editValue}
                    onChange={(e) => setEditValue(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleEdit(prop.key, prop.prop_type)
                      if (e.key === 'Escape') setEditKey(null)
                    }}
                    autoFocus
                  />
                </div>
              ) : (
                <>
                  <button
                    className="flex-1 text-left px-1 py-0.5 text-[10px] rounded hover:bg-muted truncate min-w-0"
                    onClick={() => startEdit(prop.key, prop.value)}
                    title={`${prop.key}: ${String(prop.value)} (${prop.prop_type})`}
                  >
                    {String(prop.value)}
                  </button>
                </>
              )}
              <button
                className="text-[10px] text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 shrink-0"
                onClick={() => removeProperty(selectedNodeId, prop.key)}
                title="Remove property"
              >
                &times;
              </button>
            </div>
          ))}
        </div>
      ) : (
        <p className="text-[10px] text-muted-foreground px-1">No custom properties</p>
      )}

      <div className="space-y-1 pt-1 border-t">
        <div className="flex gap-1">
          <input
            className="flex-1 px-1 py-0.5 text-[10px] border rounded bg-background min-w-0"
            placeholder="Key"
            value={newKey}
            onChange={(e) => setNewKey(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') handleAdd() }}
          />
          <select
            className="w-16 px-1 py-0.5 text-[10px] border rounded bg-background"
            value={newType}
            onChange={(e) => setNewType(e.target.value)}
          >
            <option value="text">Text</option>
            <option value="number">Num</option>
            <option value="checkbox">Bool</option>
          </select>
        </div>
        <div className="flex gap-1">
          <input
            className="flex-1 px-1 py-0.5 text-[10px] border rounded bg-background"
            placeholder={
              newType === 'text' ? 'Value'
                : newType === 'number' ? 'Number'
                : 'true/false'
            }
            value={newValue}
            onChange={(e) => setNewValue(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') handleAdd() }}
          />
          <Button
            variant="outline"
            size="sm"
            className="text-[10px] h-5 px-1"
            onClick={handleAdd}
          >
            Add
          </Button>
        </div>
      </div>
    </div>
  )
}
