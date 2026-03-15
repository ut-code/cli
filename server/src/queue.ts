import { DurableObject } from "cloudflare:workers";

// Singleton DO that manages the global queue of waiting programmers.
// State: a Map<roomId, label> stored in DO storage.
export class QueueDO extends DurableObject {
	async fetch(request: Request): Promise<Response> {
		const url = new URL(request.url);

		// GET /queue — list waiting programmers
		if (request.method === "GET" && url.pathname === "/queue") {
			const entries = await this.ctx.storage.list<string>();
			const queue = Object.fromEntries(entries);
			return Response.json(queue);
		}

		// POST /queue — register a programmer { roomId, label }
		if (request.method === "POST" && url.pathname === "/queue") {
			const { roomId, label } = await request.json<{
				roomId: string;
				label: string;
			}>();
			await this.ctx.storage.put(roomId, label);
			return new Response(null, { status: 204 });
		}

		// DELETE /queue/:roomId — remove a programmer from the queue
		const deleteMatch = url.pathname.match(/^\/queue\/(.+)$/);
		if (request.method === "DELETE" && deleteMatch) {
			await this.ctx.storage.delete(deleteMatch[1]);
			return new Response(null, { status: 204 });
		}

		return new Response("Not found", { status: 404 });
	}
}
