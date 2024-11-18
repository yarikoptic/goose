import React, { useEffect } from 'react'
import { useChat } from 'ai/react'
import { getApiUrl } from './config'
import { Card } from './components/ui/card'
import { ScrollArea } from './components/ui/scroll-area'
import GooseSplashLogo from './components/GooseSplashLogo'
import GooseMessage from './components/GooseMessage'
import UserMessage from './components/UserMessage'
import Input from './components/Input'
import Tabs from './components/Tabs'

export interface Chat {
  id: number;
  title: string;
  messages: Array<{ id: string; role: any; content: string }>;
}

export default function Chat({ chats, setChats, selectedChatId, setSelectedChatId } : { chats: Chat[], setChats: any, selectedChatId: number, setSelectedChatId: any }) {
  const chat = chats.find((c: Chat) => c.id === selectedChatId);

  const { messages, input, handleInputChange, handleSubmit } = useChat({
    api: getApiUrl("/reply"),
    initialMessages: chat.messages
  })

  const useChatData = { messages, input, handleInputChange, handleSubmit };

  useEffect(() => {
    const updatedChats = [...chats]
    updatedChats.find((c) => c.id === selectedChatId).messages = messages
    setChats(updatedChats)
  }, [messages, selectedChatId])

  return (
    <div className="flex flex-col w-screen h-screen bg-window-gradient items-center justify-center p-[10px]">
      <Tabs chats={chats} selectedChatId={selectedChatId} setSelectedChatId={setSelectedChatId} setChats={setChats} />

      <Card className="flex flex-col flex-1 h-[calc(100vh-95px)] w-full bg-card-gradient mt-0 border-none shadow-xl rounded-2xl rounded-tl-none">
        {messages.length === 0 ? (
          <div className="h-full flex items-center justify-center">
            <div className="flex items-center">
              <GooseSplashLogo />
              <span className="ask-goose-type ml-[8px]">ask<br />goose</span>
            </div>
          </div>
        ) : (
          <ScrollArea className="flex-1 px-[10px]">
            <div className="block h-10" />
            {messages.map((message) => (
              <div key={message.id}>
                {message.role === 'user' ? (
                  <UserMessage message={message} />
                ) : (
                  <GooseMessage message={message} useChatData={useChatData} />
                )}
              </div>
            ))}
            <div className="block h-10" />
          </ScrollArea>
        )}

        <Input handleSubmit={handleSubmit} handleInputChange={handleInputChange} input={input} />
      </Card>
    </div>
  )
}