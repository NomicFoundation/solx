const fs = require('fs');
const path = require('path');

// Path to reports directory (relative to this script)
const REPORTS_DIR = path.join(__dirname, 'reports');

// Read all project directories
const projects = fs.readdirSync(REPORTS_DIR)
    .filter(dir => fs.statSync(path.join(REPORTS_DIR, dir)).isDirectory());

// Initialize data object
const reportsData = {};

// Process each project
projects.forEach(project => {
    const projectDir = path.join(REPORTS_DIR, project);
    const reports = fs.readdirSync(projectDir)
        .filter(file => file.startsWith('gas-report-') && file.endsWith('.json'));
    
    // Initialize project data structure
    reportsData[project] = {
        solc: [],
        solx: []
    };
    
    // Process each report file
    reports.forEach(reportFile => {
        // Parse compiler type and version from filename
        // Example: gas-report-solc-0.8.28.json -> { type: 'solc', version: '0.8.28' }
        const match = reportFile.match(/gas-report-(solc|solx)-(.+)\.json/);
        if (!match) return;
        
        const [, compilerType, version] = match;
        const reportPath = path.join(projectDir, reportFile);
        
        try {
            const reportData = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
            // Add version information to each contract in the report
            reportData.forEach(contract => {
                contract.compiler = {
                    type: compilerType,
                    version: version
                };
            });
            
            // Add to appropriate compiler type array
            reportsData[project][compilerType].push(...reportData);
        } catch (error) {
            console.error(`Error processing ${projectDir}/${reportFile}:`, error);
        }
    });
});

// Generate JavaScript file content
const jsContent = `// Auto-generated file from build.js
const REPORTS_DATA = ${JSON.stringify(reportsData, null, 2)};`;

// Write to reports-data.js
fs.writeFileSync(
    path.join(__dirname, 'js', 'reports-data.js'),
    jsContent
);

console.log('Generated reports-data.js with data from:', Object.keys(reportsData).join(', ')); 
