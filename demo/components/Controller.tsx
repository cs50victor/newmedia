"use client"

import { useEffect } from "react"

const validCommands : Record<string, string> = {
  "W": "UP",
  "A": "LEFT",
  "S": "DOWN",
  "D": "RIGHT",
  "ARROWUP": "ZOOM-IN",
  "ARROWDOWN": "ZOOM-OUT"
}

export const Controller=({socket}: {socket: WebSocket})=>{

  const handle_keyboard_input = (e: KeyboardEvent) =>{
    const key = e.key.toUpperCase()
    if(key in validCommands){
      e.preventDefault();
      socket.send(validCommands[key])
      console.log("key down -> ", key)
    }
  }

  useEffect(() => {
    document.addEventListener("keydown", handle_keyboard_input)
    return () => document.removeEventListener("keydown", handle_keyboard_input)
  }, []);

  return null
}
