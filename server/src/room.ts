import { DurableObject } from "cloudflare:workers";

export class RoomSession extends DurableObject {
  userSocket: WebSocket | null = null;
  programmerSocket: WebSocket | null = null;

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    const role = url.pathname.endsWith("/user") ? "user" : "programmer";

    const { 0: client, 1: server } = new WebSocketPair();
    this.ctx.acceptWebSocket(server, [role]);
    return new Response(null, { status: 101, webSocket: client });
  }

  webSocketMessage(ws: WebSocket, message: string) {
    const tags = this.ctx.getTags(ws);
    if (tags.includes("user")) {
      // userからの質問をprogrammerに転送
      const programmers = this.ctx.getWebSockets("programmer");
      programmers[0]?.send(message);
    } else {
      // programmerからの回答をuserに転送
      const users = this.ctx.getWebSockets("user");
      if (message === "[DONE]") {
        users[0]?.send("[DONE]");
      } else {
        users[0]?.send(message);
      }
    }
  }

  webSocketClose(ws: WebSocket) {
    const tags = this.ctx.getTags(ws);
    if (tags.includes("user")) {
      this.programmerSocket?.close();
      this.userSocket = null;
    } else {
      this.userSocket?.close();
      this.programmerSocket = null;
    }
  }
}
