// 各パッケージのdesign/types.md(Mermaidのクラス図)と，src/の実装を照合する．
// 使い方: node scripts/check-design.mjs [パッケージのディレクトリ...]
//   (省略すると，src/とdesign/types.mdの両方を持つiterations/*/{exercise,solution}のすべて)
//
// 確かめること:
// - 図の名前空間はモジュールの名前と一致し，その中のクラスは，そのモジュールのstruct，enum，trait，typeと一致する．
// - モジュールの公開関数は，名前空間の中の<<module>>のクラス`<モジュール名>_mod`に書かれ，名前が一致する．
// - クラスの名前は名前空間の名前と重ならない(重なるとMermaidの描画が止まる)．
// テストのコード(#[cfg(test)]から後ろ)は照合しない．
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');

function packages() {
	if (process.argv.length > 2) return process.argv.slice(2).map((p) => path.resolve(p));
	const dir = path.join(root, 'iterations');
	if (!fs.existsSync(dir)) return [];
	return fs.readdirSync(dir).sort().flatMap((it) =>
		['exercise', 'solution']
			.map((kind) => path.join(dir, it, kind))
			.filter((p) => fs.existsSync(path.join(p, 'src')) && fs.existsSync(path.join(p, 'design', 'types.md'))));
}

// Mermaidのブロックから，名前空間ごとのクラスと，<<module>>のクラスのメソッド名を読む．
function readDiagram(file) {
	const text = fs.readFileSync(file, 'utf8');
	const blocks = [...text.matchAll(/```mermaid\n([\s\S]*?)\n```/g)].map((m) => m[1]);
	const classes = new Map(); // 名前 -> { namespace, module: bool, members: [] }
	const namespaces = new Set();
	for (const block of blocks) {
		let namespace = null;
		let current = null;
		for (const raw of block.split('\n')) {
			const line = raw.trim();
			let m;
			if ((m = line.match(/^namespace\s+([A-Za-z_][\w]*)\s*\{$/))) {
				namespace = m[1];
				namespaces.add(namespace);
			} else if ((m = line.match(/^class\s+([A-Za-z_]\w*)(?:~[^~]*~)?(?:\["[^"]*"\])?\s*(\{)?\s*$/))) {
				const name = m[1];
				if (!classes.has(name)) classes.set(name, { namespace, module: false, members: [] });
				else if (namespace && !classes.get(name).namespace) classes.get(name).namespace = namespace;
				current = m[2] ? classes.get(name) : null;
			} else if (line === '}') {
				if (current) current = null;
				else namespace = null;
			} else if (current) {
				if (line === '<<module>>') current.module = true;
				else current.members.push(line);
			}
		}
	}
	return { classes, namespaces };
}

function moduleName(srcDir, file) {
	const rel = path.relative(srcDir, file).replace(/\.rs$/, '').split(path.sep);
	if (rel.at(-1) === 'mod') rel.pop();
	if (rel.length === 1 && rel[0] === 'lib') return 'crate';
	return rel.join('::');
}

function rustFiles(dir) {
	return fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
		const p = path.join(dir, e.name);
		if (e.isDirectory()) return rustFiles(p);
		return e.name.endsWith('.rs') ? [p] : [];
	});
}

// モジュールごとに，定義する型と，最上位の公開関数を読む．
function readCode(srcDir) {
	const modules = new Map(); // モジュール -> { types: Set, fns: Set }
	for (const file of rustFiles(srcDir)) {
		const module = moduleName(srcDir, file);
		const text = fs.readFileSync(file, 'utf8').split(/^#\[cfg\(test\)\]/m)[0];
		const types = new Set();
		const fns = new Set();
		for (const line of text.split('\n')) {
			let m;
			if ((m = line.match(/^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait|type)\s+([A-Za-z_]\w*)/))) types.add(m[1]);
			if ((m = line.match(/^pub\s+fn\s+([A-Za-z_]\w*)/))) fns.add(m[1]);
		}
		modules.set(module, { types, fns });
	}
	return modules;
}

let errors = 0;
function report(pkg, message) {
	errors++;
	console.error(`${path.relative(root, pkg) || '.'}: ${message}`);
}

const checked = packages();
for (const pkg of checked) {
	const { classes, namespaces } = readDiagram(path.join(pkg, 'design', 'types.md'));
	const code = readCode(path.join(pkg, 'src'));

	for (const [name, info] of classes) {
		if (namespaces.has(name)) report(pkg, `クラス${name}の名前が名前空間と同じである`);
		if (info.module) {
			const module = name.replace(/_mod$/, '');
			if (!name.endsWith('_mod') || info.namespace !== module) {
				report(pkg, `<<module>>のクラス${name}は，名前空間${module}の中に${module}_modとして書く`);
				continue;
			}
			const fns = code.get(module)?.fns ?? new Set();
			const drawn = new Set(info.members.map((m) => m.match(/^[+\-#~]?\s*([A-Za-z_]\w*)\s*\(/)?.[1]).filter(Boolean));
			for (const f of drawn) if (!fns.has(f)) report(pkg, `図の${name}にある関数${f}が，${module}の公開関数にない`);
			for (const f of fns) if (!drawn.has(f)) report(pkg, `${module}の公開関数${f}が，図の${name}にない`);
			continue;
		}
		const where = info.namespace ?? '(名前空間なし)';
		if (!code.get(info.namespace ?? '')?.types.has(name)) report(pkg, `図の${where}にある型${name}が，そのモジュールのコードにない`);
	}
	for (const [module, { types, fns }] of code) {
		for (const t of types) {
			const info = classes.get(t);
			if (!info || info.namespace !== module) report(pkg, `${module}の型${t}が，図の名前空間${module}にない`);
		}
		if (fns.size > 0 && module !== 'crate' && !classes.get(`${module}_mod`)?.module) {
			report(pkg, `${module}の公開関数を書く<<module>>のクラス${module}_modが，図にない`);
		}
	}
}

console.log(`design: ${checked.length} packages, ${errors} errors`);
process.exit(errors === 0 ? 0 : 1);
