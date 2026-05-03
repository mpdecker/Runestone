import { useState, useRef, useCallback } from 'react'

interface Props {
  editor: any
}

export function AudioRecorder({ editor }: Props) {
  const [state, setState] = useState<'idle' | 'recording' | 'paused'>('idle')
  const [elapsed, setElapsed] = useState(0)
  const mediaRecorderRef = useRef<MediaRecorder | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const chunksRef = useRef<Blob[]>([])
  const timerRef = useRef<number | null>(null)

  const clearTimer = useCallback(() => {
    if (timerRef.current) { clearInterval(timerRef.current); timerRef.current = null }
  }, [])

  const startTimer = useCallback(() => {
    timerRef.current = window.setInterval(() => setElapsed((e) => e + 1), 1000)
  }, [])

  const insertAudio = useCallback((base64: string) => {
    editor?.commands?.insertContent(`<audio controls src="${base64}" class="my-2 w-full max-w-md"></audio>`)
  }, [editor])

  const finalizeRecording = useCallback(() => {
    clearTimer()
    const blob = new Blob(chunksRef.current, { type: 'audio/webm' })
    const reader = new FileReader()
    reader.onloadend = () => {
      const base64 = reader.result as string
      insertAudio(base64)
    }
    reader.readAsDataURL(blob)
    streamRef.current?.getTracks().forEach((t) => t.stop())
    streamRef.current = null
    setState('idle')
    setElapsed(0)
  }, [clearTimer, insertAudio])

  const startRecording = useCallback(async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      streamRef.current = stream
      const recorder = new MediaRecorder(stream, { mimeType: 'audio/webm' })
      chunksRef.current = []

      recorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunksRef.current.push(e.data)
      }

      recorder.onstop = () => {
        finalizeRecording()
      }

      recorder.start()
      mediaRecorderRef.current = recorder
      setState('recording')
      startTimer()
    } catch {
      alert('Microphone access denied.')
    }
  }, [startTimer, finalizeRecording])

  const stopRecording = useCallback(() => {
    mediaRecorderRef.current?.stop()
  }, [])

  const pauseRecording = useCallback(() => {
    if (mediaRecorderRef.current?.state === 'recording') {
      mediaRecorderRef.current.pause()
      setState('paused')
      clearTimer()
    }
  }, [clearTimer])

  const resumeRecording = useCallback(() => {
    if (mediaRecorderRef.current?.state === 'paused') {
      mediaRecorderRef.current.resume()
      setState('recording')
      startTimer()
    }
  }, [startTimer])

  const saveAsFile = useCallback(() => {
    const blob = new Blob(chunksRef.current, { type: 'audio/webm' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `recording-${Date.now()}.webm`
    a.click()
    URL.revokeObjectURL(url)
  }, [])

  let btnClass = 'text-muted-foreground border-border hover:bg-muted'
  let label = 'Rec'
  if (state === 'recording') {
    btnClass = 'bg-destructive text-destructive-foreground border-destructive'
    label = `REC ${elapsed}s`
  } else if (state === 'paused') {
    btnClass = 'bg-amber-500/20 text-amber-400 border-amber-500'
    label = `PAUSED ${elapsed}s`
  }

  return (
    <span className="flex items-center gap-0.5 shrink-0">
      <button
        className={`text-[10px] h-6 px-1.5 rounded border shrink-0 ${btnClass}`}
        onClick={() => {
          if (state === 'idle') startRecording()
          else if (state === 'recording') pauseRecording()
          else if (state === 'paused') stopRecording()
        }}
        title={state === 'idle' ? 'Record audio' : state === 'recording' ? 'Pause' : 'Stop'}
      >
        {label}
      </button>
      {state === 'paused' && (
        <>
          <button
            className="text-[10px] h-6 px-1 rounded border border-border text-muted-foreground hover:bg-muted shrink-0"
            onClick={resumeRecording}
            title="Resume recording"
          >
            Resume
          </button>
          <button
            className="text-[10px] h-6 px-1 rounded border border-border text-muted-foreground hover:bg-muted shrink-0"
            onClick={saveAsFile}
            title="Save as file"
          >
            Save
          </button>
        </>
      )}
    </span>
  )
}
