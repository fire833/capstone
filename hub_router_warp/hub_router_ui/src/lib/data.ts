import { writable, type Writable } from "svelte/store"

export type ApiSchema = { [idx: string]: any };

export type ApiFetchError = {
    state: "Error",
    message: string
}

export type ApiResponse = {
    state: "Success",
    data: ApiStatusData[]
}

export type ApiLoading = {
    state: "Loading"
}

export const api_data = writable<ApiLoading | ApiResponse | ApiFetchError>({ state: "Loading" });
export const  breadcrumb = writable<number[]>([0, 1, 0]);

export interface HubInfo {
    value: HubInfoValue;
}

export interface HubInfoValue {
    ready: boolean;
    message: string;
    nodes?: (NodesEntity)[] | null;
}
export interface NodesEntity {
    id: string;
    uri: string;
    maxSessions: number;
    osInfo: OsInfo;
    heartbeatPeriod: number;
    availability: string;
    version: string;
    slots?: (SlotsEntity)[] | null;
}
export interface OsInfo {
    arch: string;
    name: string;
    version: string;
}
export interface SlotsEntity {
    id: Id;
    lastStarted: string;
    session?: SessionEntity;
    stereotype: Stereotype;
}
export interface Id {
    hostId: string;
    id: string;
}
export interface Stereotype {
    browserName: string;
    browserVersion: string;
    platformName: string;
    "se:noVncPort": number;
    "se:vncEnabled": boolean;
}

export interface SessionEntity {
    capabilities: Capabilities;
    sessionId:    string;
    start:        Date;
    stereotype:   Stereotype;
    uri:          string;
}

export interface Capabilities {
    browserName:     string;
    browserVersion:  string;
    platformName:    string;
}


export type APIResult = {
    "hub_status_response": {
        Ok?: {
            response: string
        },
        Err?: {
            error: string
        }
    },
    "router_hub_state": {
        ip: string,
        port: number,
        name: string
    }
};

export type ApiStatusData = {
    "hub_status_response": HubInfo | null,
    err?: string
    "router_hub_state": {
        ip: string,
        port: number,
        name: string
    }
}

function unmarshall(result: APIResult): ApiStatusData {
    let status_response: HubInfo | null = JSON.parse(result.hub_status_response.Ok?.response ?? "null");
    
    return {
        hub_status_response: status_response,
        err: status_response === null ? result.hub_status_response.Err?.error : undefined,
        router_hub_state: result.router_hub_state
    }
}

function update_data(){
    fetch(`${window.location.origin}/api/hubs/status`, {
        method: "GET"
    })
        .then(e => {
            if (!e.ok) {
                throw new Error(`Got invalid response fetching ${e.url}:  ${e.status} ${e.statusText}`)
            }
            return e;
        })
        .then(e => e.json())
        .then(json => {
            console.log("Unmarshalled:", json.map(unmarshall));
    
            api_data.set({
                state: "Success",
                data: json.map(unmarshall)
            })
        })
        .catch(e => {
            let error: ApiFetchError = {
                state: "Error",
                message: e.toString()
            }
            api_data.set(error);
        });
}

update_data();
setInterval(update_data, 1000);


export function follow_breadcrumb(
    breadcrumb: number[],
    depth: number,
    hubs: HubInfo[]
) {
    let hub = hubs[breadcrumb[0]];
    if (depth === 0) return hub;
    let node = hub.value.nodes[breadcrumb[1]];
    if (depth === 1) return node;
    let session = node.slots[breadcrumb[2]];
    return session;
}

export function get_breadcrumb_siblings(
    breadcrumb: number[],
    depth: number,
    hubs: HubInfo[]
) {

    if (depth === 0) return hubs;
    let nodes = hubs[breadcrumb[0]].value.nodes;
    if (depth === 1) return nodes;
    let sessions = nodes[breadcrumb[1]].slots;
    return sessions;
}