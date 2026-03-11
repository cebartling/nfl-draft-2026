import { describe, it, expect } from "vitest";
import {
  extractInitialState,
  parseInitialState,
} from "../../../src/scrapers/combine/mockdraftable-parser.js";
import { CombineDataSchema } from "../../../src/types/combine.js";

describe("extractInitialState", () => {
  it("extracts JSON from basic INITIAL_STATE", () => {
    const html = `
    <html><head>
    <script>window.INITIAL_STATE = {"players": [{"firstName": "Cam", "lastName": "Ward"}]}</script>
    </head><body></body></html>`;
    const json = extractInitialState(html);
    expect(json.players).toBeDefined();
  });

  it("handles minified script", () => {
    const html = `<script>window.INITIAL_STATE={"data":{"players":[]}}</script>`;
    const json = extractInitialState(html);
    expect(json.data).toBeDefined();
  });

  it("handles multiple script tags", () => {
    const html = `
    <script>var foo = "bar";</script>
    <script>window.INITIAL_STATE = {"players": [{"firstName": "Test"}]}</script>
    <script>var baz = "qux";</script>`;
    const json = extractInitialState(html);
    expect(json.players).toBeDefined();
  });

  it("throws when INITIAL_STATE not found", () => {
    const html = "<html><body>No state here</body></html>";
    expect(() => extractInitialState(html)).toThrow();
  });
});

describe("parseInitialState — dict format (real Mockdraftable)", () => {
  const DICT_JSON = {
    measurables: {
      "1": { id: "height", key: 1, name: "Height", unit: "INCHES" },
      "3": { id: "wingspan", key: 3, name: "Wingspan", unit: "INCHES" },
      "4": { id: "arms", key: 4, name: "Arm Length", unit: "INCHES" },
      "5": { id: "hands", key: 5, name: "Hand Size", unit: "INCHES" },
      "6": { id: "10yd", key: 6, name: "10 Yard Split", unit: "SECONDS" },
      "8": { id: "40yd", key: 8, name: "40 Yard Dash", unit: "SECONDS" },
      "9": { id: "bench", key: 9, name: "Bench Press", unit: "REPS" },
      "10": { id: "vertical", key: 10, name: "Vertical Jump", unit: "INCHES" },
      "11": { id: "broad", key: 11, name: "Broad Jump", unit: "INCHES" },
    },
    players: {
      "cam-ward": {
        id: "cam-ward",
        name: "Cam Ward",
        positions: { primary: "QB", all: ["ATH", "QB"] },
        measurements: [
          { measurableKey: 8, measurement: 4.72, source: 1 },
          { measurableKey: 9, measurement: 18.0, source: 1 },
          { measurableKey: 10, measurement: 32.0, source: 1 },
          { measurableKey: 11, measurement: 108.0, source: 1 },
          { measurableKey: 4, measurement: 32.5, source: 1 },
          { measurableKey: 5, measurement: 9.75, source: 1 },
          { measurableKey: 3, measurement: 77.5, source: 1 },
        ],
      },
      "aj-haulcy": {
        id: "aj-haulcy",
        name: "AJ Haulcy",
        positions: { primary: "S", all: ["ATH", "S", "DB"] },
        measurements: [
          { measurableKey: 6, measurement: 1.62, source: 1 },
          { measurableKey: 8, measurement: 4.52, source: 1 },
        ],
      },
    },
  };

  it("parses all players from dict format", () => {
    const data = parseInitialState(DICT_JSON, 2026);
    expect(data.combine_results.length).toBe(2);
  });

  it("parses measurables by key correctly", () => {
    const data = parseInitialState(DICT_JSON, 2026);
    const cam = data.combine_results.find((e) => e.first_name === "Cam")!;
    expect(cam.forty_yard_dash).toBe(4.72);
    expect(cam.bench_press).toBe(18);
    expect(cam.vertical_jump).toBe(32.0);
    expect(cam.broad_jump).toBe(108);
    expect(cam.arm_length).toBe(32.5);
    expect(cam.hand_size).toBe(9.75);
    expect(cam.wingspan).toBe(77.5);
  });

  it("handles partial measurables", () => {
    const data = parseInitialState(DICT_JSON, 2026);
    const aj = data.combine_results.find((e) => e.first_name === "AJ")!;
    expect(aj.forty_yard_dash).toBe(4.52);
    expect(aj.ten_yard_split).toBe(1.62);
    expect(aj.bench_press).toBeNull();
  });

  it("normalizes positions (OLB → LB)", () => {
    const json = {
      players: {
        "test-player": {
          id: "test",
          name: "Test Player",
          positions: { primary: "OLB", all: ["OLB"] },
          measurements: [],
        },
      },
    };
    const data = parseInitialState(json, 2026);
    expect(data.combine_results[0].position).toBe("LB");
  });

  it("skips empty names", () => {
    const json = {
      players: {
        empty: { id: "empty", name: "", positions: { primary: "QB" }, measurements: [] },
        valid: { id: "valid", name: "Valid Player", positions: { primary: "QB" }, measurements: [] },
      },
    };
    const data = parseInitialState(json, 2026);
    expect(data.combine_results.length).toBe(1);
  });

  it("sets meta source to mockdraftable", () => {
    const data = parseInitialState(DICT_JSON, 2026);
    expect(data.meta.source).toBe("mockdraftable");
    expect(data.meta.year).toBe(2026);
  });

  it("validates against Zod schema", () => {
    const data = parseInitialState(DICT_JSON, 2026);
    const result = CombineDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });
});

describe("parseInitialState — array format", () => {
  it("parses players from array format with string measurement names", () => {
    const json = {
      players: [
        {
          firstName: "Cam",
          lastName: "Ward",
          position: "QB",
          measurements: [
            { measurementType: "40 Yard Dash", measurement: 4.72 },
            { measurementType: "Bench Press", measurement: 18.0 },
            { measurementType: "Vertical Jump", measurement: 32.0 },
          ],
        },
        {
          firstName: "Travis",
          lastName: "Hunter",
          position: "CB",
          measurements: [{ measurementType: "40 Yard Dash", measurement: 4.38 }],
        },
      ],
    };
    const data = parseInitialState(json, 2026);
    expect(data.combine_results.length).toBe(2);
    expect(data.combine_results[0].forty_yard_dash).toBe(4.72);
    expect(data.combine_results[0].bench_press).toBe(18);
  });

  it("handles nested players array", () => {
    const json = {
      data: {
        searchResults: {
          players: [
            { firstName: "Test", lastName: "Player", position: "WR", measurements: [] },
          ],
        },
      },
    };
    const data = parseInitialState(json, 2026);
    expect(data.combine_results.length).toBe(1);
    expect(data.combine_results[0].first_name).toBe("Test");
  });

  it("handles position as object with abbreviation", () => {
    const json = {
      players: [
        {
          firstName: "Test",
          lastName: "Player",
          position: { abbreviation: "DE", name: "Defensive End" },
          measurements: [],
        },
      ],
    };
    const data = parseInitialState(json, 2026);
    expect(data.combine_results[0].position).toBe("DE");
  });

  it("skips entries with empty names", () => {
    const json = {
      players: [
        { firstName: "", lastName: "", measurements: [] },
        { firstName: "Valid", lastName: "Player", position: "QB", measurements: [] },
      ],
    };
    const data = parseInitialState(json, 2026);
    expect(data.combine_results.length).toBe(1);
  });
});
