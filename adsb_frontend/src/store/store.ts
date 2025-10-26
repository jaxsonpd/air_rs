// Store for data for the project

import { Center, Position } from "../position";
import { CONFIG } from "../config";
import { Aircraft } from "../aircraft";

export class AutocompleteSuggestion {
    constructor (
    public command: string, /// the command string
    public example_value: string, /// and example value
    public similarity: number /// how similar this entry is to the search text from 0 to 1
    ) {}
}

export interface ConfigStore<T> {
  key: string; // e.g. "center", "theme", "layout"
  get(): T;
  set(newValue: T): void;

  // For search integration
  autocomplete_search(search_string: string): Array<AutocompleteSuggestion>;
  execute_search(search_string: string): boolean;

  // Optional: Reactivity hooks or event system
  onChange?(callback: () => void): () => void;
}

/**
 * Identify how similar to strings are
 * @param search the search string
 * @param command the command string
 *
 * @returns the similarity between the two strings from 0 to 1 (identical)
 */
function string_similarity(search: string, command: string): number {
    let num_correct = 0;
    for (let i = 0; i < search.length && i < command.length; i++) {
        if (search[i] == command[i]) {
            num_correct += 1;
        }
    }

    return num_correct / Math.min(search.length, command.length);

}

class CenterStore implements ConfigStore<Center> {
    private center: Center = new Center(
        CONFIG.DEFAULT_CENTER_POS,
        CONFIG.DEFAULT_CENTER_XY,
        CONFIG.DEFAULT_CENTER_PPM
    );

    private autocomplete_set: Array<{ label: string; example_value: string; }> = [
        { label: "map center geo", example_value: "-41.29, 174.78"},
        { label: "map center airport", example_value: "WLG"},
        { label: "map center icao", example_value: "8723d8"},
        { label: "testing", example_value: "8723d8"},
    ]

    key: string = "map center";

    get(): Center {
        return this.center;
    }

    set(newValue: Center): void {
        this.center = this.center;
    }

    autocomplete_search(search_string: string): Array<AutocompleteSuggestion> {
        let close_values: Array<AutocompleteSuggestion> = [];
        this.autocomplete_set.forEach((entry) => {
            const similarity = string_similarity(search_string, entry.label)
            if (similarity > 0.9) {
                close_values.push(new AutocompleteSuggestion(entry.label, entry.example_value, similarity));
            }
        });

        return close_values;
    }

    execute_search(search_string: string): boolean {
        if (search_string.startsWith("map center geo")) {
            let latitude = Number(search_string.replace("map center geo", "").split(",")[0]);
            let longitude = Number(search_string.replace("map center geo", "").split(",")[1]);
            console.log(latitude, longitude);
            this.center.pos.latitude = latitude;
            this.center.pos.longitude = longitude;
        }

        return true;
    }
}

class AircraftStore implements ConfigStore<Array<Aircraft>> {
    private aircraft_array: Array<Aircraft> = [];
    key: string = "aircraft";

    set(newValue: Aircraft[]): void {
        this.aircraft_array = newValue;
    }

    get(): Aircraft[] {
        return this.aircraft_array;
    }

    add_aircraft(aircraft: Aircraft): void {
        this.aircraft_array.concat(aircraft);
    }

    remove_aircraft(aircraft: Aircraft): void {
        this.aircraft_array = this.aircraft_array.filter(x => x != aircraft);
    }

    private autocomplete_set: Array<{ label: string; example_value: string; }> = [
        { label: "aircraft view icao", example_value: "8723c8"},
        { label: "aircraft view callsign", example_value: "ANZ100"},
    ]

    autocomplete_search(search_string: string): Array<AutocompleteSuggestion> {
        let close_values: Array<AutocompleteSuggestion> = [];
        this.autocomplete_set.forEach((entry) => {
            const similarity = string_similarity(search_string, entry.label)
            if (similarity > 0.9) {
                close_values.push(new AutocompleteSuggestion(entry.label, entry.example_value, similarity));
            }
        });

        return close_values;
    }

    execute_search(search_string: string): boolean {
        console.log(search_string);
        console.log(aircraft_store.get());
        return true;
    }

}

export let aircraft_store = new AircraftStore

export let center_store = new CenterStore;

export class SearchManager {
    private stores: ConfigStore<any>[] = [center_store, aircraft_store];

    constructor () {
    }

    autocomplete_search(search_string: string): Array<AutocompleteSuggestion> {
        return this.stores.flatMap(s => s.autocomplete_search(search_string));
    }

    execute_search(search_string: string): boolean {
        console.log("Execute %s", search_string);
        const matchingStore = this.stores.find(s => search_string.startsWith(s.key));
        const result = matchingStore?.execute_search(search_string);
        if (result == null) {
            return false;
        } else {
            return result;
        }
    }

}
