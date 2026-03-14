import { DurableObject } from "cloudflare:workers";

export interface QueueEntry {
  roomId: string;
  name: string;
}

export class ProgrammerQueue extends DurableObject {
  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    const method = request.method;
    const pathname = url.pathname;

    if (method === "POST" && pathname === "/queue/register") {
      return this.handleRegister(request);
    } else if (method === "DELETE" && pathname.match(/^\/queue\/[^/]+$/)) {
      const roomId = pathname.split("/").pop();
      return this.handleDelete(roomId!);
    } else if (method === "GET" && pathname === "/queue") {
      return this.handleGetQueue();
    }

    return new Response("Not found", { status: 404 });
  }

  private async handleRegister(request: Request): Promise<Response> {
    try {
      const body = await request.json<QueueEntry>();
      if (!body.roomId || typeof body.roomId !== "string" ||
          !body.name   || typeof body.name   !== "string") {
        return new Response("Missing or invalid fields: roomId and name are required", {
          status: 400,
        });
      }
      await this.ctx.storage.put(`entry:${body.roomId}`, {
        roomId: body.roomId,
        name: body.name,
      } satisfies QueueEntry);
      return new Response(JSON.stringify({ roomId: body.roomId }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    } catch (e) {
      return new Response("Invalid request body", { status: 400 });
    }
  }

  private async handleDelete(roomId: string): Promise<Response> {
    await this.ctx.storage.delete(`entry:${roomId}`);
    return new Response(JSON.stringify({ success: true }), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  }

  private async handleGetQueue(): Promise<Response> {
    const map = await this.ctx.storage.list<QueueEntry>({ prefix: "entry:" });
    const entries = Array.from(map.values());
    return new Response(JSON.stringify(entries), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  }
}
