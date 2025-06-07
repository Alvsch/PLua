--!strict

declare pumpkin: {
    log: Log,
    server: Server,
    events: Events,
}

export type Events = {
    register_listener: (event_type: string, callback: (...any) -> ()) -> string,
    unregister_listener: (event_type: string, listener_id: string) -> boolean,
}

export type Server = {
    broadcast_message: (message: string) -> (),
}

export type Log = {
    info: (message: string) -> (),
    warn: (message: string) -> (),
    error: (message: string) -> (),
    debug: (message: string) -> (),
}

export type Plugin = {
    name: string,
    description: string,
    version: string,
    author: string,
    on_enable: () -> (),
    on_disable: () -> (),
}