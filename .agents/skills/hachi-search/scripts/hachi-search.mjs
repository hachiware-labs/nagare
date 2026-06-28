#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const PROVIDERS = ["local", "wiki-garden", "tavily", "exa", "firecrawl"];
const ENRICHERS = ["firecrawl"];
const MERGE_STRATEGIES = ["ranked", "provider-order", "dedupe-only", "agent-sort"];
const SKILL_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const DEFAULT_CONFIG_PATH = resolve(SKILL_DIR, "configs", "default.json");
const PRESETS = {
  local: {
    tools: ["local"],
    order: ["local"],
    merge: "ranked",
  },
  full: {
    tools: [...PROVIDERS],
    order: [...PROVIDERS],
    merge: "ranked",
  },
  "調べて": {
    merge: "ranked",
  },
};

function parseArgs(argv) {
  const args = {
    roots: ["."],
    tools: [...PROVIDERS],
    order: [...PROVIDERS],
    merge: "ranked",
    limit: 8,
    enrich: [],
    enrichLimit: 5,
    json: false,
    ...readConfigIfExists(DEFAULT_CONFIG_PATH),
  };
  const explicit = {
    tools: false,
    order: false,
    merge: false,
  };
  const positional = [];

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = argv[i + 1];
    if (arg === "--json") {
      args.json = true;
    } else if (arg === "--query" || arg === "-q") {
      args.query = next;
      i += 1;
    } else if (arg === "--roots") {
      args.roots = splitList(next);
      i += 1;
    } else if (arg === "--tools") {
      args.tools = splitList(next);
      explicit.tools = true;
      i += 1;
    } else if (arg === "--enrich") {
      args.enrich = splitList(next);
      i += 1;
    } else if (arg === "--enrich-limit") {
      args.enrichLimit = Number.parseInt(next, 10);
      i += 1;
    } else if (arg === "--set-tools") {
      args.action = "set-tools";
      args.setTools = splitList(next);
      i += 1;
    } else if (arg === "--order") {
      args.order = splitList(next);
      explicit.order = true;
      i += 1;
    } else if (arg === "--merge") {
      args.merge = next;
      explicit.merge = true;
      i += 1;
    } else if (arg === "--set-merge") {
      args.action = "set-merge";
      args.setMerge = next;
      i += 1;
    } else if (arg === "--limit") {
      args.limit = Number.parseInt(next, 10);
      i += 1;
    } else if (arg === "--config") {
      const config = JSON.parse(readFileSync(resolve(next), "utf8"));
      Object.assign(args, config);
      i += 1;
    } else if (arg === "--preset" || arg === "--mode") {
      applyPreset(args, next, explicit);
      i += 1;
    } else if (PRESETS[arg]) {
      positional.push(arg);
    } else {
      positional.push(arg);
    }
  }

  if (positional.length) {
    const settings = parseJapaneseSettings(positional.join(" "));
    if (settings) {
      args.action = settings.action;
      if (settings.tools) args.setTools = settings.tools;
      if (settings.merge) args.setMerge = settings.merge;
    }
    const normalized = normalizeJapaneseShorthand(positional);
    if (normalized.preset) applyPreset(args, normalized.preset, explicit);
    if (!args.query) args.query = normalized.query;
  }

  if (!["set-tools", "set-merge"].includes(args.action) && !args.query) {
    throw new Error("Missing required --query");
  }
  args.roots = normalizeList(args.roots);
  args.tools = normalizeList(args.tools);
  args.enrich = validateEnrichers(args.enrich);
  args.order = normalizeList(args.order).filter((tool) => args.tools.includes(tool));
  args.merge = validateMerge(args.merge);
  args.limit = Number.isFinite(args.limit) && args.limit > 0 ? args.limit : 8;
  args.enrichLimit = Number.isFinite(args.enrichLimit) && args.enrichLimit > 0 ? args.enrichLimit : 5;
  if (args.action === "set-tools") args.setTools = validateProviders(args.setTools);
  if (args.action === "set-merge") args.setMerge = validateMerge(args.setMerge);
  return args;
}

function readConfigIfExists(path) {
  if (!existsSync(path)) return {};
  return JSON.parse(readFileSync(path, "utf8"));
}

function applyPreset(args, presetName, explicit = {}) {
  const preset = PRESETS[presetName];
  if (!preset) {
    throw new Error(`Unknown preset: ${presetName}`);
  }
  if (!explicit.tools && preset.tools) args.tools = preset.tools;
  if (!explicit.order && preset.order) args.order = preset.order;
  if (!explicit.merge && preset.merge) args.merge = preset.merge;
}

function normalizeJapaneseShorthand(positional) {
  const words = [...positional];
  if (PRESETS[words[0]]) {
    const [preset, ...queryWords] = words;
    return { preset, query: queryWords.join(" ") };
  }
  if (words.at(-1) === "を調べて") {
    words.pop();
    return { preset: "調べて", query: words.join(" ") };
  }
  if (words.at(-1) === "調べて") {
    words.pop();
    if (words.at(-1) === "を") words.pop();
    return { preset: "調べて", query: words.join(" ") };
  }
  return { preset: undefined, query: words.join(" ") };
}

function parseJapaneseSettings(text) {
  const compact = text.replace(/\s+/g, "");
  const toolsMatch = compact.match(/(?:hachi-searchの)?使用する検索を(.+?)に設定して/);
  if (toolsMatch) return { action: "set-tools", tools: splitList(toolsMatch[1]) };
  const mergeMatch = compact.match(/(?:hachi-searchの)?マージ方法を(.+?)に設定して/);
  if (mergeMatch) return { action: "set-merge", merge: mergeMatch[1] };
  return undefined;
}

function validateProviders(providers) {
  const normalized = normalizeList(providers);
  const unknown = normalized.filter((provider) => !PROVIDERS.includes(provider));
  if (unknown.length) {
    throw new Error(`Unknown provider(s): ${unknown.join(", ")}`);
  }
  if (!normalized.length) {
    throw new Error("At least one provider is required");
  }
  return normalized;
}

function validateEnrichers(enrichers) {
  const normalized = normalizeList(enrichers);
  const unknown = normalized.filter((enricher) => !ENRICHERS.includes(enricher));
  if (unknown.length) {
    throw new Error(`Unknown enricher(s): ${unknown.join(", ")}`);
  }
  return normalized;
}

function validateMerge(strategy) {
  strategy = normalizeMergeName(strategy);
  if (!MERGE_STRATEGIES.includes(strategy)) {
    throw new Error(`Unknown merge strategy: ${strategy}`);
  }
  return strategy;
}

function normalizeMergeName(strategy) {
  const normalized = String(strategy || "").trim();
  const compact = normalized.replace(/\s+/g, "").toLowerCase();
  if (["provider順", "provider-order", "providerorder"].includes(compact)) return "provider-order";
  if (["llmでソート", "llmソート", "agent-sort", "agentsort"].includes(compact)) return "agent-sort";
  return normalized;
}

function saveDefaultTools(tools) {
  const config = {
    ...readConfigIfExists(DEFAULT_CONFIG_PATH),
    roots: ["."],
    tools,
    order: tools,
    merge: "ranked",
    limit: 8,
  };
  mkdirSync(dirname(DEFAULT_CONFIG_PATH), { recursive: true });
  writeFileSync(DEFAULT_CONFIG_PATH, `${JSON.stringify(config, null, 2)}\n`, "utf8");
  return config;
}

function saveDefaultMerge(merge) {
  const config = {
    roots: ["."],
    tools: [...PROVIDERS],
    order: [...PROVIDERS],
    limit: 8,
    ...readConfigIfExists(DEFAULT_CONFIG_PATH),
    merge,
  };
  mkdirSync(dirname(DEFAULT_CONFIG_PATH), { recursive: true });
  writeFileSync(DEFAULT_CONFIG_PATH, `${JSON.stringify(config, null, 2)}\n`, "utf8");
  return config;
}

function splitList(value) {
  return String(value || "")
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function normalizeList(value) {
  if (Array.isArray(value)) return value.map(String).map((item) => item.trim()).filter(Boolean);
  return splitList(value);
}

function hasCommand(command) {
  const check = process.platform === "win32" ? "where.exe" : "command";
  const args = process.platform === "win32" ? [command] : ["-v", command];
  return spawnSync(check, args, { encoding: "utf8" }).status === 0;
}

async function runLocal(query, roots, limit) {
  if (!hasCommand("rg")) {
    return { provider: "local", status: "skipped", reason: "rg is not available", results: [] };
  }

  const rgArgs = [
    "--line-number",
    "--no-heading",
    "--hidden",
    "--glob",
    "!.git",
    "--glob",
    "!node_modules",
    "--glob",
    "!dist",
    "--glob",
    "!.next",
    query,
    ...roots,
  ];
  const result = spawnSync("rg", rgArgs, { encoding: "utf8", maxBuffer: 1024 * 1024 * 8 });
  const lines = result.stdout.split(/\r?\n/).filter(Boolean).slice(0, limit);
  return {
    provider: "local",
    status: result.status === 0 || lines.length ? "ok" : "ok",
    results: lines.map((line, index) => {
      const parsed = line.match(/^(.*?):(\d+):(.*)$/);
      return {
        provider: "local",
        title: parsed ? parsed[1] : "local match",
        source: parsed ? `${parsed[1]}:${parsed[2]}` : line,
        url: parsed ? parsed[1] : undefined,
        snippet: parsed ? parsed[3].trim() : line,
        score: 100 - index,
      };
    }),
  };
}

async function runWikiGarden(query, limit) {
  if (!hasCommand("wiki-garden")) {
    return { provider: "wiki-garden", status: "skipped", reason: "wiki-garden CLI is not available", results: [] };
  }

  const attempts = [
    ["query", query, "--limit", String(limit), "--json"],
    ["query", query, "--limit", String(limit)],
    ["search", query, "--limit", String(limit)],
  ];

  for (const args of attempts) {
    const result = spawnSync("wiki-garden", args, { encoding: "utf8", maxBuffer: 1024 * 1024 * 8 });
    if (result.status === 0 && result.stdout.trim()) {
      return normalizeWikiGarden(result.stdout, limit);
    }
  }

  return { provider: "wiki-garden", status: "skipped", reason: "wiki-garden command did not return results", results: [] };
}

function normalizeWikiGarden(output, limit) {
  try {
    const parsed = JSON.parse(output);
    const items = Array.isArray(parsed) ? parsed : parsed.results || parsed.items || [];
    return {
      provider: "wiki-garden",
      status: "ok",
      results: items.slice(0, limit).map((item, index) => ({
        provider: "wiki-garden",
        title: item.title || item.path || item.name || "wiki-garden result",
        source: item.path || item.url || item.id || "wiki-garden",
        url: item.url || item.path,
        snippet: item.snippet || item.summary || item.content || item.text || "",
        score: 95 - index,
      })),
    };
  } catch {
    return {
      provider: "wiki-garden",
      status: "ok",
      results: output.split(/\r?\n/).filter(Boolean).slice(0, limit).map((line, index) => ({
        provider: "wiki-garden",
        title: "wiki-garden result",
        source: "wiki-garden",
        snippet: line,
        score: 95 - index,
      })),
    };
  }
}

async function postJson(provider, url, headers, body) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json", ...headers },
    body: JSON.stringify(body),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`${provider} HTTP ${response.status}: ${text.slice(0, 300)}`);
  }
  return JSON.parse(text);
}

async function runTavily(query, limit) {
  const key = process.env.TAVILY_API_KEY;
  if (!key) return { provider: "tavily", status: "skipped", reason: "TAVILY_API_KEY is not set", results: [] };

  const data = await postJson(
    "tavily",
    "https://api.tavily.com/search",
    { Authorization: `Bearer ${key}` },
    { query, search_depth: "basic", max_results: limit, include_answer: false },
  );

  return {
    provider: "tavily",
    status: "ok",
    results: (data.results || []).slice(0, limit).map((item, index) => ({
      provider: "tavily",
      title: item.title || item.url,
      source: item.url,
      url: item.url,
      snippet: item.content || item.snippet || "",
      score: item.score ?? 80 - index,
    })),
  };
}

async function runExa(query, limit) {
  const key = process.env.EXA_API_KEY;
  if (!key) return { provider: "exa", status: "skipped", reason: "EXA_API_KEY is not set", results: [] };

  const data = await postJson(
    "exa",
    "https://api.exa.ai/search",
    { "x-api-key": key },
    { query, numResults: limit, contents: { highlights: true, text: true } },
  );

  return {
    provider: "exa",
    status: "ok",
    results: (data.results || []).slice(0, limit).map((item, index) => ({
      provider: "exa",
      title: item.title || item.url,
      source: item.url,
      url: item.url,
      snippet: Array.isArray(item.highlights) && item.highlights.length ? item.highlights.join(" ") : item.text || item.summary || "",
      publishedDate: item.publishedDate,
      score: 78 - index,
    })),
  };
}

async function runFirecrawl(query, limit) {
  const key = process.env.FIRECRAWL_API_KEY;
  if (!key) return { provider: "firecrawl", status: "skipped", reason: "FIRECRAWL_API_KEY is not set", results: [] };

  const data = await postJson(
    "firecrawl",
    "https://api.firecrawl.dev/v2/search",
    { Authorization: `Bearer ${key}` },
    { query, limit, sources: ["web"] },
  );

  const webResults = Array.isArray(data.data?.web) ? data.data.web : Array.isArray(data.data) ? data.data : [];
  return {
    provider: "firecrawl",
    status: "ok",
    results: webResults.slice(0, limit).map((item, index) => ({
      provider: "firecrawl",
      title: item.title || item.url,
      source: item.url,
      url: item.url,
      snippet: item.description || item.markdown || item.content || "",
      score: 76 - index,
    })),
  };
}

async function scrapeFirecrawl(url) {
  const key = process.env.FIRECRAWL_API_KEY;
  if (!key) throw new Error("FIRECRAWL_API_KEY is not set");

  const data = await postJson(
    "firecrawl",
    "https://api.firecrawl.dev/v2/scrape",
    { Authorization: `Bearer ${key}` },
    {
      url,
      formats: ["markdown"],
      onlyMainContent: true,
      removeBase64Images: true,
      timeout: 30000,
    },
  );
  const content = data.data?.markdown || data.data?.summary || "";
  return {
    title: data.data?.metadata?.title,
    sourceURL: data.data?.metadata?.sourceURL || data.data?.metadata?.url,
    content,
  };
}

function dedupeKey(result) {
  if (result.provider === "local") return String(result.source || result.url || result.title || "").toLowerCase();
  return String(result.url || result.source || result.title || "").replace(/#.*$/, "").replace(/\/$/, "").toLowerCase();
}

async function mergeResults(providerResults, strategy, order) {
  const seen = new Set();
  const flat = [];

  for (const provider of order) {
    const block = providerResults.find((entry) => entry.provider === provider);
    for (const result of block?.results || []) {
      const key = dedupeKey(result);
      if (!key || seen.has(key)) continue;
      seen.add(key);
      flat.push(result);
    }
  }

  if (strategy === "provider-order" || strategy === "dedupe-only") return { results: flat };
  if (strategy === "agent-sort") {
    return {
      results: flat,
      note: "agent-sort requested; sort these deduped results in the calling LLM/skill response",
    };
  }
  return { results: flat.sort((a, b) => (b.score || 0) - (a.score || 0)) };
}

async function enrichResults(results, enrichers, limit) {
  if (!enrichers.includes("firecrawl")) return { results };

  const enriched = [];
  const notes = [];
  let scraped = 0;

  for (const result of results) {
    const url = result.url || result.source;
    if (scraped >= limit || !/^https?:\/\//i.test(String(url || ""))) {
      enriched.push(result);
      continue;
    }

    try {
      const scrape = await scrapeFirecrawl(url);
      scraped += 1;
      enriched.push({
        ...result,
        title: result.title || scrape.title,
        enrichedBy: "firecrawl",
        enrichedSource: scrape.sourceURL || url,
        content: scrape.content,
        snippet: scrape.content ? String(scrape.content).replace(/\s+/g, " ").slice(0, 600) : result.snippet,
      });
    } catch (error) {
      notes.push(`firecrawl enrich failed for ${url}: ${error.message}`);
      enriched.push(result);
    }
  }

  return { results: enriched, note: notes.join("; ") || undefined };
}

async function runProvider(provider, args) {
  try {
    if (provider === "local") return await runLocal(args.query, args.roots, args.limit);
    if (provider === "wiki-garden") return await runWikiGarden(args.query, args.limit);
    if (provider === "tavily") return await runTavily(args.query, args.limit);
    if (provider === "exa") return await runExa(args.query, args.limit);
    if (provider === "firecrawl") return await runFirecrawl(args.query, args.limit);
    return { provider, status: "skipped", reason: "unknown provider", results: [] };
  } catch (error) {
    return { provider, status: "error", reason: error.message, results: [] };
  }
}

function printText(output) {
  console.log(`# hachi-search: ${output.query}`);
  console.log("");
  console.log("## Results");
  if (!output.results.length) {
    console.log("No results.");
  }
  for (const [index, result] of output.results.entries()) {
    console.log(`${index + 1}. [${result.provider}] ${result.title || result.source}`);
    console.log(`   Source: ${result.source || result.url || "unknown"}`);
    if (result.snippet) console.log(`   ${String(result.snippet).replace(/\s+/g, " ").slice(0, 300)}`);
  }
  console.log("");
  console.log("## Provider Notes");
  for (const provider of output.providers) {
    const note = provider.reason ? ` - ${provider.reason}` : "";
    console.log(`- ${provider.provider}: ${provider.status}${note}`);
  }
  if (output.mergeNote) {
    console.log(`- merge: ${output.mergeNote}`);
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.action === "set-tools") {
    const config = saveDefaultTools(args.setTools);
    console.log(`# hachi-search settings updated`);
    console.log(`Tools: ${config.tools.join(",")}`);
    console.log(`Config: ${DEFAULT_CONFIG_PATH}`);
    return;
  }
  if (args.action === "set-merge") {
    const config = saveDefaultMerge(args.setMerge);
    console.log(`# hachi-search settings updated`);
    console.log(`Merge: ${config.merge}`);
    console.log(`Config: ${DEFAULT_CONFIG_PATH}`);
    return;
  }
  const providers = [];

  for (const provider of args.order) {
    if (!PROVIDERS.includes(provider)) {
      providers.push({ provider, status: "skipped", reason: "unsupported provider", results: [] });
      continue;
    }
    providers.push(await runProvider(provider, args));
  }

  const merged = await mergeResults(providers, args.merge, args.order);
  const enriched = await enrichResults(merged.results, args.enrich, args.enrichLimit);
  const notes = [merged.note, enriched.note].filter(Boolean);
  const output = {
    query: args.query,
    roots: args.roots.map((root) => (existsSync(root) ? resolve(root) : root)),
    merge: args.merge,
    enrich: args.enrich,
    mergeNote: notes.join("; ") || undefined,
    providers,
    results: enriched.results,
  };

  if (args.json) {
    console.log(JSON.stringify(output, null, 2));
  } else {
    printText(output);
  }
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
