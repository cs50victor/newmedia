import WebSocketExample from "~/components/WebSocketExample";

export default function Page() {
  return (
    <div className="min-h-dvh prose w-full mx-auto">
      <div className="prose flex flex-col items-center justify-center">
        <h1 className="text-center">Curr Image from New Media</h1>
        <WebSocketExample/>
      </div>
    </div>
  )
}
