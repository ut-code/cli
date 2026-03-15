import { DurableObject } from "cloudflare:workers";

export class RoomSession extends DurableObject {
	async fetch(request: Request): Promise<Response> {
		const url = new URL(request.url);
		const role = url.pathname.endsWith("/programmer") ? "programmer" : "client";

		const { 0: client, 1: server } = new WebSocketPair();
		this.ctx.acceptWebSocket(server, [role]);
		return new Response(null, { status: 101, webSocket: client });
	}

	webSocketMessage(ws: WebSocket, message: string) {
		const tags = this.ctx.getTags(ws);
		if (tags.includes("client")) {
			// クライアントからの質問をプログラマーに転送
			const programmers = this.ctx.getWebSockets("programmer");
			programmers[0]?.send(message);
		} else {
			// プログラマーからの回答をクライアントに転送
			const clients = this.ctx.getWebSockets("client");
			clients[0]?.send(message);
		}
	}

	webSocketClose(ws: WebSocket) {
		const tags = this.ctx.getTags(ws);
		if (tags.includes("client")) {
			this.ctx.getWebSockets("programmer")[0]?.close();
		} else {
			this.ctx.getWebSockets("client")[0]?.close();
		}
	}
}
