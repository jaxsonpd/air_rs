// Store for data for the project

import { Center } from "../position";
import { CONFIG } from "../config"; 

export interface ConfigStore<T> {
  key: string; // e.g. "center", "theme", "layout"
  get(): T;
  set(newValue: T): void;

  // For search integration
  autocomplete_search(search_string: string): Array<{ label: string; example_value: string}>;
  execute_search(search_string: string): boolean;

  // Optional: Reactivity hooks or event system
  onChange?(callback: () => void): () => void;
}

class CenterStore implements ConfigStore<Center> {
    private data: Center = new Center(
        CONFIG.DEFAULT_CENTER_POS,
        CONFIG.DEFAULT_CENTER_XY,
        CONFIG.DEFAULT_CENTER_PPM
    );

    key: string = "map center";

    get(): Center {
        return this.data;
    }

    set(newValue: Center): void {
        this.data = this.data;
    }

    autocomplete_search(search_string: string): Array<{ label: string; example_value: string; }> {
        return [
            { label: "map center geo", example_value: "-41.29, 174.78"}
        ];
    }

    execute_search(search_string: string): boolean {
        
        return true;
    }
}

class SearchManager {
    
}
