import { writable, type Writable } from "svelte/store"

export type ApiSchema = {[idx: string]: any};

export type ApiFetchError = {
    is_error: true,
    message: string
}

export const api_data = writable<undefined | ApiSchema | ApiFetchError>(undefined);

fetch(`${window.location.origin}/api/hubs`)
    .then(e => {
        if(!e.ok){
            throw new Error(`Got invalid response fetching ${e.url}:  ${e.status} ${e.statusText}`)
        }
        return e;
    })
    .then(e => e.json())
    .then(json => {
        console.log("Got json: ", json);
        api_data.set(json);
    })
    .catch(e => {
        let error: ApiFetchError = {
            is_error: true,
            message: e.toString()
        }
        api_data.set(error);
    });
