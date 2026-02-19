#!/usr/bin/env node

const requiredMajor = 24;

const [majorStr] = process.versions.node.split('.');
const major = Number(majorStr);

if (!Number.isInteger(major) || major !== requiredMajor) {
	console.error(
		`Pocket Ratings frontend tests must be run with Node.js ${requiredMajor}. ` +
			`Current Node.js version: ${process.versions.node}. ` +
			'Please switch to Node 24 (for example via nvm) before running `bun run test`.'
	);
	process.exit(1);
}

