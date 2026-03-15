import { Hono } from "hono";
import { QueueDO } from "./queue";
import { RoomSession } from "./room";

export { QueueDO, RoomSession };

type Bindings = {
	ROOM: DurableObjectNamespace;
	QUEUE: DurableObjectNamespace;
};

const app = new Hono<{ Bindings: Bindings }>();

// Helper: get the singleton QueueDO stub
function queueStub(c: { env: Bindings }) {
	const id = c.env.QUEUE.idFromName("global");
	return c.env.QUEUE.get(id);
}

// GET /queue — list waiting programmers { [roomId]: label }
app.get("/queue", (c) => {
	return queueStub(c).fetch(new Request("http://do/queue"));
});

// POST /queue — programmer registers { label }; creates a room and returns roomId
app.post("/queue", async (c) => {
	const { label } = await c.req.json<{ label: string }>();
	const roomId = c.env.ROOM.newUniqueId().toString();
	await queueStub(c).fetch(
		new Request("http://do/queue", {
			method: "POST",
			body: JSON.stringify({ roomId, label }),
		}),
	);
	return c.json({ roomId });
});

// DELETE /queue/:roomId — remove a programmer from the queue
app.delete("/queue/:roomId", (c) => {
	const roomId = c.req.param("roomId");
	return queueStub(c).fetch(
		new Request(`http://do/queue/${roomId}`, { method: "DELETE" }),
	);
});

// WebSocket: programmer connects to their room
app.get("/rooms/:id/programmer", (c) => {
	const id = c.env.ROOM.idFromString(c.req.param("id"));
	return c.env.ROOM.get(id).fetch(c.req.raw);
});

// WebSocket: client connects to a chosen room
app.get("/rooms/:id/client", (c) => {
	const id = c.env.ROOM.idFromString(c.req.param("id"));
	return c.env.ROOM.get(id).fetch(c.req.raw);
});

export default app;
