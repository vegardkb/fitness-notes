import { readFileSync, writeFileSync, existsSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptDir, "..");
const require = createRequire(import.meta.url);

function resolvePkgDir(name, from) {
    try {
        return path.dirname(
            require.resolve(path.join(name, "package.json"), { paths: [from] }),
        );
    } catch {
        try {
            const entry = require.resolve(name, { paths: [from] });
            let dir = path.dirname(entry);
            while (dir !== path.dirname(dir)) {
                if (existsSync(path.join(dir, "package.json"))) return dir;
                dir = path.dirname(dir);
            }
        } catch {
            return null;
        }
        return null;
    }
}

function readPkg(dir) {
    return JSON.parse(readFileSync(path.join(dir, "package.json"), "utf8"));
}

function licenseString(l) {
    if (Array.isArray(l)) return l.join(" OR ");
    return typeof l === "string" ? l : "(none)";
}

const npm = new Map();
function collectNpm(dir, deps) {
    for (const name of Object.keys(deps || {})) {
        if (npm.has(name)) continue;
        const d = resolvePkgDir(name, dir);
        if (!d) continue;
        let j;
        try {
            j = readPkg(d);
        } catch {
            continue;
        }
        npm.set(j.name, { version: j.version, license: licenseString(j.license), dir: d });
        collectNpm(d, { ...j.dependencies, ...j.peerDependencies });
    }
}

const pkgJson = readPkg(root);
collectNpm(root, pkgJson.dependencies);

const registryRoots = (() => {
    const src = path.join(os.homedir(), ".cargo", "registry", "src");
    if (!existsSync(src)) return [];
    return readdirSync(src).map((d) => path.join(src, d));
})();

function crateDir(name, version) {
    const base = version.split("+")[0];
    for (const r of registryRoots) {
        for (const v of [version, base]) {
            const p = path.join(r, `${name}-${v}`);
            if (existsSync(p)) return p;
        }
    }
    for (const r of registryRoots) {
        if (!existsSync(r)) continue;
        let entries;
        try {
            entries = readdirSync(r);
        } catch {
            continue;
        }
        const hit = entries.find((d) => d.startsWith(`${name}-${base}`));
        if (hit) return path.join(r, hit);
    }
    return null;
}

const meta = JSON.parse(
    spawnSync("cargo", ["metadata", "--format-version", "1"], {
        cwd: path.join(root, "src-tauri"),
        encoding: "utf8",
        maxBuffer: 1 << 26,
    }).stdout,
);
const ws = new Set(meta.workspace_members);
const crates = meta.packages
    .filter((p) => !ws.has(p.id))
    .map((p) => ({
        name: p.name,
        version: p.version,
        license: licenseString(p.license),
        dir: crateDir(p.name, p.version),
    }))
    .sort((a, b) => a.name.localeCompare(b.name));

const texts = {};
function normalize(t) {
    return t
        .replace(/\r\n/g, "\n")
        .split("\n")
        .filter((l) => !/^Copyright/i.test(l))
        .join("\n")
        .replace(/\n{3,}/g, "\n\n")
        .trim();
}

function detect(content) {
    if (/Mozilla Public License(,)? Version 2/i.test(content)) return "MPL-2.0";
    if (/SIL Open Font License/i.test(content)) return "OFL-1.1";
    if (/UNICODE LICENSE V3|Unicode License Agreement/i.test(content)) return "Unicode-3.0";
    if (/Apache License\s*\n?\s*Version 2/i.test(content)) return "Apache-2.0";
    if (/Permission is hereby granted, free of charge/i.test(content)) {
        return /shall be included in all copies|substantial portions/i.test(content)
            ? "MIT"
            : "MIT-0";
    }
    if (/Permission to use, copy, modify, and\/or distribute/i.test(content)) return "ISC";
    if (/Redistribution and use in source and binary forms/i.test(content)) {
        return /Neither the name/i.test(content) ? "BSD-3-Clause" : "BSD-2-Clause";
    }
    if (/free and unencumbered software released into the public domain/i.test(content))
        return "Unlicense";
    if (/CC0 1\.0 Universal/i.test(content)) return "CC0-1.0";
    if (/provided ['"]as-is['"], without any express/i.test(content)) return "Zlib";
    if (/Blue Oak Model License/i.test(content)) return "BlueOak-1.0.0";
    if (/Boost Software License/i.test(content)) return "BSL-1.0";
    return null;
}

function scanTexts(dirs) {
    for (const d of dirs) {
        if (!d || !existsSync(d)) continue;
        let files;
        try {
            files = readdirSync(d);
        } catch {
            continue;
        }
        for (const f of files) {
            if (!/^LICENSE/i.test(f)) continue;
            let content;
            try {
                content = readFileSync(path.join(d, f), "utf8");
            } catch {
                continue;
            }
            const id = detect(content);
            if (id && !texts[id]) texts[id] = normalize(content);
        }
    }
}

scanTexts([...npm.values().map((p) => p.dir), ...crates.map((c) => c.dir)]);

function copyrightLines(dir) {
    if (!dir || !existsSync(dir)) return [];
    let files;
    try {
        files = readdirSync(dir);
    } catch {
        return [];
    }
    const lines = [];
    for (const f of files) {
        if (!/^LICENSE/i.test(f) || f.endsWith(".spdx")) continue;
        let content;
        try {
            content = readFileSync(path.join(dir, f), "utf8");
        } catch {
            continue;
        }
        for (const m of content.matchAll(/^Copyright[^\n]*/gm)) {
            lines.push(m[0].trim());
            if (lines.length >= 3) return lines;
        }
    }
    return lines;
}

function stripOuterParens(e) {
    let s = e.trim();
    while (s.startsWith("(") && s.endsWith(")")) {
        let depth = 0;
        let wraps = true;
        for (let i = 0; i < s.length; i++) {
            if (s[i] === "(") depth++;
            else if (s[i] === ")") {
                depth--;
                if (depth === 0 && i !== s.length - 1) {
                    wraps = false;
                    break;
                }
            }
        }
        if (!wraps) break;
        s = s.slice(1, -1).trim();
    }
    return s;
}

function splitTop(e, sep) {
    const parts = [];
    let depth = 0;
    let cur = "";
    for (let i = 0; i < e.length; i++) {
        const ch = e[i];
        if (ch === "(") depth++;
        if (ch === ")") depth--;
        if (depth === 0 && e.startsWith(sep, i)) {
            parts.push(cur.trim());
            cur = "";
            i += sep.length - 1;
            continue;
        }
        cur += ch;
    }
    parts.push(cur.trim());
    return parts.filter(Boolean);
}

function optionText(opt) {
    const base = opt.split(" WITH ")[0].trim().replace(/\+$/, "");
    if (texts[base]) return texts[base];
    const aliases = {
        "BSD-3": "BSD-3-Clause",
        "BSD-2": "BSD-2-Clause",
        "Apache-2": "Apache-2.0",
        MPL: "MPL-2.0",
    };
    if (aliases[base] && texts[aliases[base]]) return texts[aliases[base]];
    return null;
}

function normalizeExpr(expr) {
    return expr.trim().replace(/\s*\/\s*/g, " OR ");
}

function textFor(expr) {
    const e = stripOuterParens(normalizeExpr(expr));
    const orParts = e.includes(" OR ") ? splitTop(e, " OR ") : [];
    if (orParts.length > 1) {
        for (const part of orParts) {
            const t = textFor(stripOuterParens(part));
            if (t) return t;
        }
        return null;
    }
    const andParts = e.includes(" AND ") ? splitTop(e, " AND ") : [];
    if (andParts.length > 1) {
        const parts = andParts.map((p) => textFor(stripOuterParens(p)));
        if (parts.some((t) => !t)) return null;
        return parts.join("\n\n---\n\n");
    }
    return optionText(e);
}

const groups = new Map();
function add(category, name, version, license, dir) {
    const key = `${category}\u0000${normalizeExpr(license)}`;
    if (!groups.has(key)) {
        groups.set(key, {
            category,
            license: normalizeExpr(license),
            packages: [],
            copyrights: new Set(),
        });
    }
    const g = groups.get(key);
    g.packages.push(version ? `${name} ${version}` : name);
    for (const c of copyrightLines(dir)) g.copyrights.add(c);
}

for (const p of npm.values()) {
    add("JavaScript packages", p.name, p.version, p.license, p.dir);
}
for (const c of crates) {
    add("Rust crates", c.name, c.version, c.license, c.dir);
}

const fontPkgs = [];
const fontCopyrights = new Set();
let fontText = "";
for (const font of ["dm-sans", "source-serif-4"]) {
    const pkgName = `@fontsource/${font}`;
    const dir = resolvePkgDir(pkgName, root);
    if (!dir) continue;
    const j = readPkg(dir);
    fontPkgs.push(`${pkgName} ${j.version}`);
    const licenseFile = readdirSync(dir).find((f) => /^LICENSE/i.test(f));
    const fullText = licenseFile
        ? readFileSync(path.join(dir, licenseFile), "utf8")
        : "";
    for (const c of fullText.match(/^Copyright[^\n]*/gm) || []) {
        fontCopyrights.add(c);
    }
    if (!fontText) fontText = fullText.trim();
}
if (fontPkgs.length) {
    groups.set("Fonts\u0000OFL-1.1", {
        category: "Fonts",
        license: "OFL-1.1",
        packages: fontPkgs,
        copyrights: [...fontCopyrights],
        text: fontText,
    });
}

const categoryOrder = { Fonts: 0, "JavaScript packages": 1, "Rust crates": 2 };
const unknown = [];
const out = [...groups.values()]
    .map((g) => {
        let text = g.text ?? textFor(g.license);
        if (!text) {
            unknown.push(`${g.category}: ${g.license}`);
            text = `License text for ${g.license} is not bundled. See https://spdx.org/licenses/.`;
        }
        return {
            category: g.category,
            license: g.license,
            packages: g.packages.sort(),
            copyrights: [...g.copyrights].sort().slice(0, 120),
            text,
        };
    })
    .sort(
        (a, b) =>
            categoryOrder[a.category] - categoryOrder[b.category] ||
            a.license.localeCompare(b.license),
        );

const ts = `export type LicenseGroup = {
    category: string;
    license: string;
    packages: string[];
    copyrights: string[];
    text: string;
};

export const licenseGroups: LicenseGroup[] = ${JSON.stringify(out)};
`;
writeFileSync(path.join(root, "src", "lib", "licenses.ts"), ts);

console.log(`npm packages: ${npm.size}`);
console.log(`rust crates: ${crates.length}`);
console.log(`groups: ${out.length}`);
console.log(`texts available: ${Object.keys(texts).sort().join(", ")}`);
if (unknown.length) console.log(`UNKNOWN LICENSES: ${unknown.join("; ")}`);
