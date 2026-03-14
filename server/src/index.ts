import { Hono } from "hono";
import { RoomSession } from "./room";
import { ProgrammerQueue } from "./queue";

export { RoomSession, ProgrammerQueue };

type Bindings = { ROOM: DurableObjectNamespace; QUEUE: DurableObjectNamespace };

const app = new Hono<{ Bindings: Bindings }>();

app.post("/rooms", (c) => {
  const id = c.env.ROOM.newUniqueId();
  return c.json({ roomId: id.toString() });
});

app.get("/rooms/:id/user", (c) => {
  const id = c.env.ROOM.idFromString(c.req.param("id"));
  const stub = c.env.ROOM.get(id);
  return stub.fetch(c.req.raw);
});

app.get("/rooms/:id/programmer", (c) => {
  const id = c.env.ROOM.idFromString(c.req.param("id"));
  const stub = c.env.ROOM.get(id);
  return stub.fetch(c.req.raw);
});

// Queue endpoints
app.post("/queue/register", (c) => {
  const queueId = c.env.QUEUE.idFromName("global");
  const stub = c.env.QUEUE.get(queueId);
  return stub.fetch(c.req.raw);
});

app.delete("/queue/:roomId", (c) => {
  const queueId = c.env.QUEUE.idFromName("global");
  const stub = c.env.QUEUE.get(queueId);
  return stub.fetch(c.req.raw);
});

app.get("/queue", (c) => {
  const queueId = c.env.QUEUE.idFromName("global");
  const stub = c.env.QUEUE.get(queueId);
  return stub.fetch(c.req.raw);
});

export default app;
