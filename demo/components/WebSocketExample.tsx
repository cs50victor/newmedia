"use client"

import { useEffect, useState } from 'react';
import { Canvas } from './Canvas';
import { Controller } from './Controller';

export interface ServerWSResponse {
  image: string
  dimension: [number, number]
}

export default function WebSocketExample({port = 8080}:{port?:number}){
  const [imgMetadata, setImageMetadata] = useState<ServerWSResponse>();
  const [socket, setSocket] = useState<WebSocket>()

  useEffect(() => {
    const socket = new WebSocket(`ws://localhost:${port}`)

    socket.onopen = () => {
      setSocket(socket)
      console.log('WebSocket Open');
      socket.send("hello");
    };

    socket.onclose = () => {
      console.log('WebSocket Close');
    };

    socket.onerror = (error) => {
      console.error("WebSocket Error:", error);
    };

    socket.onmessage = (event) => {
      try {
        const data : ServerWSResponse = JSON.parse(event.data);
        console.log("WebSocket Message:", data);
        if (data.image) {
          setImageMetadata(data)
        }
      } catch (e) {
        console.error("Error parsing the WebSocket response:", e);
      }
    };

    return () => {
      socket?.close();
    };
  }, []);

  return (
    <div>
      { imgMetadata ? (
        <Canvas img_metadata={imgMetadata} />
      ) : <p className='text-xl font-semibold'>NEW MEDIA | trying to connect to server...</p>
      }
      {socket ? <Controller socket={socket}/> : null}
    </div>
  )
};
