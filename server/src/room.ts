import { DurableObject } from "cloudflare:workers";

export class RoomSession extends DurableObject {
  async fetch(request: Request): Promise<Response> {
    // Reject non-WebSocket requests
    if (request.headers.get("Upgrade") !== "websocket") {
      return new Response("Expected WebSocket upgrade", { status: 426 });
    }

    const url = new URL(request.url);
    const isUser = url.pathname.endsWith("/user");
    const isProgrammer = url.pathname.endsWith("/programmer");

    if (!isUser && !isProgrammer) {
      return new Response("Invalid room endpoint", { status: 400 });
    }

    const role = isUser ? "user" : "programmer";

    // Enforce one connection per role
    if (this.ctx.getWebSockets(role).length > 0) {
      return new Response(`A ${role} is already connected to this room`, {
        status: 409,
      });
    }

    const { 0: client, 1: server } = new WebSocketPair();
    this.ctx.acceptWebSocket(server, [role]);
    return new Response(null, { status: 101, webSocket: client });
  }

  webSocketMessage(ws: WebSocket, message: string) {
    const tags = this.ctx.getTags(ws);
    if (tags.includes("user")) {
      // Forward user's question to the programmer
      const programmers = this.ctx.getWebSockets("programmer");
      programmers[0]?.send(message);
    } else {
      // Forward programmer's response (including [DONE]) to the user
      const users = this.ctx.getWebSockets("user");
      users[0]?.send(message);
    }
  }

  webSocketClose(ws: WebSocket) {
    const tags = this.ctx.getTags(ws);
    if (tags.includes("user")) {
      // User disconnected — notify and close the programmer side
      for (const sock of this.ctx.getWebSockets("programmer")) {
        sock.close(1000, "User disconnected");
      }
    } else {
      // Programmer disconnected — notify and close the user side
      for (const sock of this.ctx.getWebSockets("user")) {
        sock.close(1000, "Programmer disconnected");
      }
    }
  }

  webSocketError(ws: WebSocket, error: unknown) {
    const tags = this.ctx.getTags(ws);
    const role = tags.includes("user") ? "user" : "programmer";
    console.error(`WebSocket error on ${role} connection:`, error);

    // Close the other side so neither party hangs
    const other = role === "user" ? "programmer" : "user";
    for (const sock of this.ctx.getWebSockets(other)) {
      sock.close(1011, `${role} connection error`);
    }
  }
}
