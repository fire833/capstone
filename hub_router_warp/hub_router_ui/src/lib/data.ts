import { writable, type Writable } from "svelte/store"

export type ApiSchema = { [idx: string]: any };

export type ApiFetchError = {
    state: "Error",
    message: string
}

export type ApiResponse = {
    state: "Success",
    data: APIStatusData[]
}

export type ApiLoading = {
    state: "Loading"
}

export const api_data = writable<ApiLoading | ApiResponse | ApiFetchError>({ state: "Loading" });
export const  breadcrumb = writable<number[]>([0, 1, 0]);


export type APIResult = {
    "hub_status_response": {
        Ok?: {
            response: string
        },
        Err?: {
            error: string
        }
    },
    "router_hub_state": RouterHubState
};

export interface APIStatusData {
    hub_status_response: HubInfo;
    err: string;
    router_hub_state:    RouterHubState;
}

export interface HubStatusResponse {
    value: HubInfo;
}

export interface HubInfo {
    ready:   boolean;
    message: string;
    nodes:   Node[];
}

export interface Node {
    id:              string;
    uri:             string;
    maxSessions:     number;
    osInfo:          OSInfo;
    heartbeatPeriod: number;
    availability:    string;
    version:         string;
    slots:           Slot[];
}

export interface OSInfo {
    arch:    string;
    name:    string;
    version: string;
}

export interface Slot {
    id:          ID;
    lastStarted: Date;
    session:     null;
    stereotype:  SlotStereotype;
}

export interface ID {
    hostId: string;
    id:     string;
}

export interface SlotStereotype {
    browserName:     string;
    browserVersion:  string;
    platformName:    string;
    "se:noVncPort":  number;
    "se:vncEnabled": boolean;
}

export interface RouterHubState {
    meta:  Meta;
    state: State;
}

export interface Meta {
    name: string;
    url:  string;
    uuid: string;
}

export interface State {
    fullness:                         number;
    stereotypes:                      StereotypeElement[];
    readiness:                        string;
    consecutive_healthcheck_failures: number;
}

export interface StereotypeElement {
    browserName:  string;
    platformName: string;
}


function unmarshall(result: APIResult): APIStatusData {
    let status_response: HubStatusResponse | null = JSON.parse(result.hub_status_response.Ok?.response ?? "null");
    
    return {
        hub_status_response: status_response?.value,
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
    let node = hub.nodes[breadcrumb[1]];
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
    let nodes = hubs[breadcrumb[0]].nodes;
    if (depth === 1) return nodes;
    let sessions = nodes[breadcrumb[1]].slots;
    return sessions;
}