declare namespace wasm_bindgen {
	/* tslint:disable */
	/* eslint-disable */
	/**
	*/
	export function greet(): void;
	/**
	* @param {any} boards
	* @param {string} lms1
	* @param {string} lms2
	* @param {number} min_num_lm
	* @param {number} max_num_lm
	* @param {any} weights
	* @param {number} booster
	* @returns {BoardoptResult}
	*/
	export function boardopt(boards: any, lms1: string, lms2: string, min_num_lm: number, max_num_lm: number, weights: any, booster: number): BoardoptResult;
	/**
	*/
	export class BoardoptResult {
	  free(): void;
	/**
	*/
	  boosted: string;
	/**
	*/
	  placed: string;
	/**
	*/
	  score: number;
	}
	
}

declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_boardoptresult_free: (a: number) => void;
  readonly __wbg_get_boardoptresult_score: (a: number) => number;
  readonly __wbg_set_boardoptresult_score: (a: number, b: number) => void;
  readonly __wbg_get_boardoptresult_placed: (a: number, b: number) => void;
  readonly __wbg_set_boardoptresult_placed: (a: number, b: number, c: number) => void;
  readonly __wbg_get_boardoptresult_boosted: (a: number, b: number) => void;
  readonly __wbg_set_boardoptresult_boosted: (a: number, b: number, c: number) => void;
  readonly greet: () => void;
  readonly boardopt: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
declare function wasm_bindgen (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
