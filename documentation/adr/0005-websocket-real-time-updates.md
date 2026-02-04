# ADR 0005: WebSocket for Real-Time Draft Updates

## Status

Accepted

## Context

The NFL Draft Simulator requires real-time updates to all connected clients when draft events occur (picks made, trades executed, etc.). We needed to choose a mechanism for real-time communication between the server and clients.

Requirements:

- Real-time updates to all clients in a draft session
- Low latency (< 100ms) for draft pick notifications
- Support for multiple concurrent draft sessions
- Bidirectional communication (client actions, server updates)
- Connection resilience (handle disconnects, reconnections)
- Scalability (100+ concurrent clients per draft session)

Options considered:

1. **WebSocket**: Full-duplex communication over a single TCP connection
2. **Server-Sent Events (SSE)**: Server-to-client streaming over HTTP
3. **Long Polling**: Client repeatedly polls server for updates
4. **HTTP/2 Server Push**: Server pushes resources to client
5. **GraphQL Subscriptions**: GraphQL over WebSocket

## Decision

We will use **WebSocket** for real-time communication between the backend and frontend.

Implementation:

- **Backend**: tokio-tungstenite WebSocket library with Axum
- **Frontend**: Native WebSocket API with auto-reconnection wrapper
- **Protocol**: JSON-based message format with Zod validation
- **Connection Management**: In-memory connection manager using DashMap

## Consequences

### Positive

- **True Real-Time**: Full-duplex communication enables instant updates
- **Low Latency**: Single persistent connection eliminates HTTP overhead
- **Bidirectional**: Server can push updates AND receive client actions
- **Efficient**: One connection handles all communication vs polling with multiple requests
- **Native Browser Support**: WebSocket API is built into all modern browsers
- **Scalability**: Tokio async runtime handles thousands of concurrent connections
- **Type Safety**: JSON messages validated with Zod schemas on both ends

### Negative

- **Complexity**: More complex than simple HTTP polling
- **Connection Management**: Need to handle disconnects, reconnects, stale connections
- **Stateful**: Server must maintain connection state (vs stateless HTTP)
- **Load Balancing**: Requires sticky sessions or shared state for horizontal scaling
- **Debugging**: Harder to debug than HTTP requests (need special tools)
- **Firewall Issues**: Some corporate firewalls block WebSocket connections

### Neutral

- **Protocol Choice**: JSON is human-readable but larger than binary formats (protobuf, msgpack)
- **Browser Support**: WebSocket is well-supported but needs fallback for very old browsers
- **Testing**: Requires special test utilities for WebSocket connections

## Architecture

### Backend WebSocket Server

```rust
// websocket/src/connection_manager.rs
pub struct ConnectionManager {
    // session_id -> Vec<client_id>
    sessions: DashMap<Uuid, Vec<Uuid>>,
    // client_id -> WebSocket sender
    clients: DashMap<Uuid, mpsc::UnboundedSender<Message>>,
}

impl ConnectionManager {
    pub async fn broadcast(&self, session_id: Uuid, message: Message) {
        if let Some(clients) = self.sessions.get(&session_id) {
            for client_id in clients.iter() {
                if let Some(sender) = self.clients.get(client_id) {
                    let _ = sender.send(message.clone());
                }
            }
        }
    }
}
```

### Frontend WebSocket Client

```typescript
// lib/api/websocket.ts
export class WebSocketClient {
  private ws: WebSocket | null = null;
  private state = $state<WebSocketState>(WebSocketState.Disconnected);
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  connect(url: string) {
    this.ws = new WebSocket(url);
    this.setupHandlers();
  }

  private setupHandlers() {
    this.ws!.onopen = () => {
      this.state = WebSocketState.Connected;
      this.reconnectAttempts = 0;
    };

    this.ws!.onmessage = (event) => {
      const message = this.parseMessage(event.data);
      this.handleMessage(message);
    };

    this.ws!.onclose = () => {
      this.state = WebSocketState.Disconnected;
      this.attemptReconnect();
    };

    this.ws!.onerror = (error) => {
      logger.error("WebSocket error:", error);
    };
  }

  private async attemptReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      return;
    }

    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
    this.reconnectAttempts++;

    await new Promise((resolve) => setTimeout(resolve, delay));
    this.connect(this.url);
  }
}
```

## Message Protocol

### Message Format

All WebSocket messages use JSON with a consistent structure:

```typescript
// Client -> Server
interface ClientMessage {
  type: "join_session" | "make_pick" | "propose_trade";
  payload: unknown;
}

// Server -> Client
interface ServerMessage {
  type: "draft_pick" | "trade_executed" | "session_update";
  payload: unknown;
}
```

### Message Validation

Messages are validated using Zod schemas:

```typescript
// lib/types/websocket.ts
const DraftPickMessageSchema = z.object({
  type: z.literal("draft_pick"),
  payload: z.object({
    pick: DraftPickSchema,
    session_id: z.string().uuid(),
  }),
});
```

### Example Message Flow

1. **Client joins session**:

   ```json
   {
     "type": "join_session",
     "payload": { "session_id": "..." }
   }
   ```

2. **Server acknowledges**:

   ```json
   {
     "type": "session_joined",
     "payload": { "session_id": "...", "current_pick": 1 }
   }
   ```

3. **Client makes pick**:

   ```json
   {
     "type": "make_pick",
     "payload": { "player_id": "...", "team_id": "..." }
   }
   ```

4. **Server broadcasts to all clients in session**:
   ```json
   {
     "type": "draft_pick",
     "payload": { "pick": { ... }, "next_pick": 2 }
   }
   ```

## Connection Management

### Heartbeat / Ping-Pong

To detect stale connections:

- Client sends ping every 30 seconds
- Server responds with pong
- Client disconnects if no pong received within 5 seconds
- Server closes connections that don't send ping after 60 seconds

```typescript
// Client-side heartbeat
private startHeartbeat() {
  this.heartbeatInterval = setInterval(() => {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'ping' }));
    }
  }, 30000);
}
```

### Reconnection Strategy

Exponential backoff with jitter:

- Attempt 1: 1 second
- Attempt 2: 2 seconds
- Attempt 3: 4 seconds
- Attempt 4: 8 seconds
- Attempt 5: 16 seconds
- Max: 30 seconds

After 5 failed attempts, show error to user and stop reconnecting.

### Session Re-joining

When reconnecting, client re-joins the session:

1. Client reconnects to WebSocket
2. Client sends `join_session` message with session_id
3. Server sends current session state (current pick, recent picks)
4. Client updates UI to match server state

## Scalability Considerations

### Current Implementation (Single Server)

- In-memory connection manager (DashMap)
- All connections to a single server instance
- Simple but not horizontally scalable

### Future Horizontal Scaling

If we need to scale beyond a single server:

1. **Redis Pub/Sub**: Use Redis for session coordination
   - Each server subscribes to session channels
   - When a pick is made, publish to Redis
   - All servers receive the message and broadcast to their clients

2. **Sticky Sessions**: Route all clients of a session to the same server
   - Use load balancer with session affinity
   - Simpler than Redis but limits failover

3. **Dedicated WebSocket Servers**: Separate WebSocket servers from API servers
   - API servers handle HTTP requests
   - WebSocket servers handle only real-time connections
   - API servers publish events to WebSocket servers via message queue

## Testing Strategy

### Backend Tests

Mock WebSocket connections for testing broadcast logic:

```rust
#[tokio::test]
async fn test_broadcast_to_session() {
    let manager = ConnectionManager::new();
    let session_id = Uuid::new_v4();

    // Add mock clients
    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();

    manager.add_client(session_id, client1_id, tx1);
    manager.add_client(session_id, client2_id, tx2);

    // Broadcast message
    manager.broadcast(session_id, Message::DraftPick { ... }).await;

    // Verify both clients received message
    assert!(rx1.try_recv().is_ok());
    assert!(rx2.try_recv().is_ok());
}
```

### Frontend Tests

Mock WebSocket for testing client logic:

```typescript
// Mock WebSocket for testing
class MockWebSocket {
  readyState = WebSocket.OPEN;

  send(data: string) {
    // Capture sent messages
  }

  simulateMessage(data: string) {
    this.onmessage?.({ data });
  }
}

describe("WebSocketClient", () => {
  it("should reconnect on disconnect", async () => {
    const client = new WebSocketClient();
    client.connect("ws://test");

    // Simulate disconnect
    client.ws.simulateClose();

    // Verify reconnection attempt
    await new Promise((resolve) => setTimeout(resolve, 1100));
    expect(client.reconnectAttempts).toBe(1);
  });
});
```

## Alternatives Considered

### Server-Sent Events (SSE)

**Pros**: Simpler than WebSocket, automatic reconnection, works over HTTP
**Cons**: One-way only (server -> client), client must use HTTP for actions
**Rejected**: Need bidirectional communication for client actions (make pick, propose trade)

### Long Polling

**Pros**: Works everywhere (even old browsers), simple concept
**Cons**: High latency, inefficient (many HTTP requests), server load
**Rejected**: Latency requirements make this unsuitable

### GraphQL Subscriptions

**Pros**: Type-safe, integrates with GraphQL API, well-defined schema
**Cons**: Uses WebSocket anyway, adds GraphQL complexity, overkill for simple messages
**Rejected**: Don't need GraphQL complexity for simple message passing

## References

- [WebSocket MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/)
- [WebSocket Best Practices](https://www.ably.com/topic/websockets)
