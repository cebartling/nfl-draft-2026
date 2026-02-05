/**
 * Script to download NFL team logos from ESPN CDN
 *
 * Usage: npx tsx scripts/download-team-logos.ts
 *
 * This script fetches team logo URLs from nflverse data and downloads
 * PNG logos from ESPN CDN to the static/logos/teams/ directory.
 */

import * as fs from 'fs';
import * as path from 'path';
import * as https from 'https';

// NFL team data with ESPN logo URLs
// Source: nflverse teams_colors_logos.csv
const NFL_TEAMS = [
	{ abbreviation: 'ARI', name: 'Arizona Cardinals' },
	{ abbreviation: 'ATL', name: 'Atlanta Falcons' },
	{ abbreviation: 'BAL', name: 'Baltimore Ravens' },
	{ abbreviation: 'BUF', name: 'Buffalo Bills' },
	{ abbreviation: 'CAR', name: 'Carolina Panthers' },
	{ abbreviation: 'CHI', name: 'Chicago Bears' },
	{ abbreviation: 'CIN', name: 'Cincinnati Bengals' },
	{ abbreviation: 'CLE', name: 'Cleveland Browns' },
	{ abbreviation: 'DAL', name: 'Dallas Cowboys' },
	{ abbreviation: 'DEN', name: 'Denver Broncos' },
	{ abbreviation: 'DET', name: 'Detroit Lions' },
	{ abbreviation: 'GB', name: 'Green Bay Packers' },
	{ abbreviation: 'HOU', name: 'Houston Texans' },
	{ abbreviation: 'IND', name: 'Indianapolis Colts' },
	{ abbreviation: 'JAX', name: 'Jacksonville Jaguars' },
	{ abbreviation: 'KC', name: 'Kansas City Chiefs' },
	{ abbreviation: 'LAC', name: 'Los Angeles Chargers' },
	{ abbreviation: 'LAR', name: 'Los Angeles Rams' },
	{ abbreviation: 'LV', name: 'Las Vegas Raiders' },
	{ abbreviation: 'MIA', name: 'Miami Dolphins' },
	{ abbreviation: 'MIN', name: 'Minnesota Vikings' },
	{ abbreviation: 'NE', name: 'New England Patriots' },
	{ abbreviation: 'NO', name: 'New Orleans Saints' },
	{ abbreviation: 'NYG', name: 'New York Giants' },
	{ abbreviation: 'NYJ', name: 'New York Jets' },
	{ abbreviation: 'PHI', name: 'Philadelphia Eagles' },
	{ abbreviation: 'PIT', name: 'Pittsburgh Steelers' },
	{ abbreviation: 'SEA', name: 'Seattle Seahawks' },
	{ abbreviation: 'SF', name: 'San Francisco 49ers' },
	{ abbreviation: 'TB', name: 'Tampa Bay Buccaneers' },
	{ abbreviation: 'TEN', name: 'Tennessee Titans' },
	{ abbreviation: 'WAS', name: 'Washington Commanders' },
];

function getEspnLogoUrl(abbreviation: string): string {
	return `https://a.espncdn.com/i/teamlogos/nfl/500/${abbreviation.toLowerCase()}.png`;
}

function downloadFile(url: string, destPath: string): Promise<void> {
	return new Promise((resolve, reject) => {
		const file = fs.createWriteStream(destPath);

		https
			.get(url, (response) => {
				// Handle redirects
				if (response.statusCode === 301 || response.statusCode === 302) {
					const redirectUrl = response.headers.location;
					if (redirectUrl) {
						file.close();
						fs.unlinkSync(destPath);
						downloadFile(redirectUrl, destPath).then(resolve).catch(reject);
						return;
					}
				}

				if (response.statusCode !== 200) {
					file.close();
					fs.unlinkSync(destPath);
					reject(new Error(`Failed to download: HTTP ${response.statusCode}`));
					return;
				}

				response.pipe(file);

				file.on('finish', () => {
					file.close();
					resolve();
				});
			})
			.on('error', (err) => {
				file.close();
				fs.unlink(destPath, () => {}); // Delete the file on error
				reject(err);
			});
	});
}

async function main() {
	const scriptDir = path.dirname(new URL(import.meta.url).pathname);
	const outputDir = path.join(scriptDir, '..', 'static', 'logos', 'teams');

	// Ensure output directory exists
	if (!fs.existsSync(outputDir)) {
		fs.mkdirSync(outputDir, { recursive: true });
	}

	console.log('Downloading NFL team logos...');
	console.log(`Output directory: ${outputDir}\n`);

	let successCount = 0;
	let failCount = 0;

	for (const team of NFL_TEAMS) {
		const logoUrl = getEspnLogoUrl(team.abbreviation);
		const outputPath = path.join(outputDir, `${team.abbreviation}.png`);

		process.stdout.write(`Downloading ${team.abbreviation} (${team.name})... `);

		try {
			await downloadFile(logoUrl, outputPath);
			console.log('✓');
			successCount++;
		} catch (error) {
			console.log(`✗ (${error instanceof Error ? error.message : 'Unknown error'})`);
			failCount++;
		}
	}

	console.log(`\nComplete! ${successCount} succeeded, ${failCount} failed.`);

	if (failCount > 0) {
		process.exit(1);
	}
}

main().catch((error) => {
	console.error('Script failed:', error);
	process.exit(1);
});
