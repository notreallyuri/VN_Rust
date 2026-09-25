export type Note = {
  severity: "error" | "warning";
  line: number;
  message: string;
};

export type Step = {
  kind: string;
  speaker?: string;
  text?: string;
  detail?: string;
  taken?: string;
};

export type Choice = {
  index: number;
  text: string;
  enabled: boolean;
  reason?: string;
};

export type Answer = {
  ok: boolean;
  notes: Note[];
  listing: string;
  counts: { scenes: number; instructions: number };
  steps: Step[];
  choices: Choice[];
  ended: boolean;
  stopped?: string;
};

type Exports = {
  memory: WebAssembly.Memory;
  novn_alloc: (len: number) => number;
  novn_free: (ptr: number, len: number) => void;
  novn_run: (ptr: number, len: number) => number;
};

const WASM = `${process.env.NEXT_PUBLIC_BASE_PATH ?? ""}/novn-playground.wasm`;

let loading: Promise<Exports> | null = null;

function load(): Promise<Exports> {
  const wanted = fetch(WASM).then(async (response) => {
    if (!response.ok) throw new Error(`${WASM}: ${response.status}`);
    const { instance } = await WebAssembly.instantiate(
      await response.arrayBuffer(),
      {},
    );
    return instance.exports as unknown as Exports;
  });
  loading = wanted;
  return wanted;
}

export function ready(): Promise<Exports> {
  return loading ?? load();
}

export async function ask(
  source: string,
  picks: number[] = [],
): Promise<Answer> {
  let wasm = await ready();
  try {
    return call(wasm, source, picks);
  } catch (first) {
    loading = null;
    wasm = await ready();
    try {
      return call(wasm, source, picks);
    } catch {
      throw first;
    }
  }
}

function call(wasm: Exports, source: string, picks: number[]): Answer {
  const request = new TextEncoder().encode(JSON.stringify({ source, picks }));
  const at = wasm.novn_alloc(request.length);
  if (at === 0) throw new Error("the playground could not allocate");

  let answer: number | null = null;
  try {
    new Uint8Array(wasm.memory.buffer, at, request.length).set(request);
    answer = wasm.novn_run(at, request.length);
    const size = new DataView(wasm.memory.buffer).getUint32(answer, true);
    const json = new TextDecoder().decode(
      new Uint8Array(wasm.memory.buffer, answer + 4, size),
    );
    wasm.novn_free(answer, 4 + size);
    answer = null;
    return JSON.parse(json) as Answer;
  } finally {
    try {
      wasm.novn_free(at, request.length);
      if (answer !== null) {
        const size = new DataView(wasm.memory.buffer).getUint32(answer, true);
        wasm.novn_free(answer, 4 + size);
      }
    } catch {
      loading = null;
    }
  }
}
