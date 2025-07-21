// Add global charts array for chart management
let charts = [];

// Function to show loading state
function showLoading() {
    document.getElementById('loading').classList.add('active');
    document.getElementById('error').classList.remove('active');
}

// Function to hide loading state
function hideLoading() {
    document.getElementById('loading').classList.remove('active');
}

// Function to show error message
function showError(message) {
    const errorDiv = document.getElementById('error');
    errorDiv.textContent = message;
    errorDiv.classList.add('active');
    hideLoading();
}

// Function to read the gas report data
function readGasReports(projectName) {
    try {
        const projectData = REPORTS_DATA[projectName];
        if (!projectData) {
            throw new Error(`No data available for project: ${projectName}`);
        }
        return {
            solcData: projectData.solc,
            solxData: projectData.solx
        };
    } catch (error) {
        showError(`Error reading gas reports: ${error.message}`);
        console.error('Error reading gas reports:', error);
        return null;
    }
}

// Color schemes for different compiler versions
const colorSchemes = {
    solc: {
        base: '#9C27B0', // Original purple
        getColor: (index, total) => {
            const baseHue = 291; // Purple hue
            const saturation = 76; // Fixed saturation
            // Adjust lightness from 45% to 65% based on index
            const lightness = 45 + (index * 10);
            return `hsl(${baseHue}, ${saturation}%, ${lightness}%)`;
        },
        getBorderColor: (color) => {
            // Extract HSL values from the color string
            const match = color.match(/hsl\((\d+),\s*(\d+)%,\s*(\d+)%\)/);
            if (!match) return color;
            const [, h, s, l] = match;
            // Return a darker version of the same color
            return `hsl(${h}, ${s}%, ${Math.max(20, parseInt(l) - 10)}%)`;
        }
    },
    solx: {
        base: '#2196F3', // Base blue
        getColor: (index, total) => {
            const baseHue = 207; // Blue hue
            const saturation = 90; // Fixed saturation
            // Adjust lightness from 45% to 65% based on index
            const lightness = 45 + (index * 10);
            return `hsl(${baseHue}, ${saturation}%, ${lightness}%)`;
        },
        getBorderColor: (color) => {
            // Extract HSL values from the color string
            const match = color.match(/hsl\((\d+),\s*(\d+)%,\s*(\d+)%\)/);
            if (!match) return color;
            const [, h, s, l] = match;
            // Return a darker version of the same color
            return `hsl(${h}, ${s}%, ${Math.max(20, parseInt(l) - 10)}%)`;
        }
    }
};

// Helper functions for color manipulation
function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16)
    } : null;
}

function rgbToHsl(rgb) {
    const r = rgb.r / 255;
    const g = rgb.g / 255;
    const b = rgb.b / 255;
    
    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    let h, s, l = (max + min) / 2;

    if (max === min) {
        h = s = 0;
    } else {
        const d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
        switch (max) {
            case r: h = (g - b) / d + (g < b ? 6 : 0); break;
            case g: h = (b - r) / d + 2; break;
            case b: h = (r - g) / d + 4; break;
        }
        h /= 6;
    }

    return [h * 360, s * 100, l * 100];
}

// Function to calculate gas differences for a single contract
function calculateContractDiff(contractData) {
    const results = [];
    let methodCount = 0;

    // Find compiler settings objects and contract objects
    const settingsObjects = contractData.filter(obj => obj.compilerSettings && !obj.contract);
    const contractObjects = contractData.filter(obj => obj.contract);

    // Group data by function name across all compiler versions
    const functionData = {};
    
    contractObjects.forEach(version => {
        Object.entries(version.functions).forEach(([funcName, data]) => {
            if (!functionData[funcName]) {
                functionData[funcName] = [];
            }
            // Add compiler settings to the version data
            const settings = settingsObjects.find(s => s.compiler.type === version.compiler.type && 
                s.compiler.version === version.compiler.version)?.compilerSettings || {};
            functionData[funcName].push({
                compiler: version.compiler,
                gas: data.mean,
                compilerSettings: settings
            });
        });
    });

    // Find solx default version (without via-ir)
    const solxDefault = contractObjects.find(obj => 
        obj.compiler.type === 'solx' && 
        !obj.compiler.version.includes('via-ir')
    );

    if (!solxDefault) {
        console.error('No default solx version found');
        return null;
    }

    // Calculate differences for each function
    Object.entries(functionData).forEach(([funcName, versions]) => {
        const solxDefaultVersion = versions.find(v => 
            v.compiler.type === 'solx' && 
            !v.compiler.version.includes('via-ir')
        );

        if (solxDefaultVersion) {
            const baseGas = solxDefaultVersion.gas;
            results.push({
                name: funcName,
                versions: versions.filter(v => 
                    v === solxDefaultVersion || // Include solx default
                    v.compiler.type === 'solc' || // Include all solc versions
                    (v.compiler.type === 'solx' && v.compiler.version.includes('via-ir')) // Include solx via-ir
                ).map(v => ({
                    ...v,
                    baseGas,
                    difference: ((v.gas - baseGas) / baseGas * 100).toFixed(2)
                }))
            });
            methodCount++;
        }
    });

    // Get deployment info for all versions
    const deploymentInfo = contractObjects.map(version => {
        const settings = settingsObjects.find(s => 
            s.compiler.type === version.compiler.type && 
            s.compiler.version === version.compiler.version
        )?.compilerSettings || {};
        return {
            compiler: version.compiler,
            gas: version.deployment.gas,
            compilerSettings: settings
        };
    });

    // Improved deployment sorting: solc-opt first, then other solc, then all solx
    const sortedDeployment = [...deploymentInfo].sort((a, b) => {
        // solc-opt (not via-ir) first
        const isSolcOpt = v => v.compiler.type === 'solc' && v.compiler.version.includes('opt') && !v.compiler.version.includes('via-ir');
        const isSolc = v => v.compiler.type === 'solc';
        const isSolx = v => v.compiler.type === 'solx';
        if (isSolcOpt(a) && !isSolcOpt(b)) return -1;
        if (!isSolcOpt(a) && isSolcOpt(b)) return 1;
        if (isSolc(a) && isSolc(b)) return 0;
        if (isSolc(a) && isSolx(b)) return -1;
        if (isSolx(a) && isSolc(b)) return 1;
        return 0;
    });

    return {
        results,
        deployment: sortedDeployment,
        methodCount,
        contractName: solxDefault.contract,
        contractObjects // for use in updateSummary
    };
}

// Function to update the dashboard
function updateChart(data) {
    if (!data || !Array.isArray(data) || data.length === 0) {
        showError('No valid data to display');
        return;
    }

    // Clear existing charts
    charts.forEach(chart => chart.destroy());
    charts = [];
    
    // Get the chart container
    const container = document.getElementById('chartContainer');
    if (!container) {
        console.error('Chart container not found');
        return;
    }
    
    // Clear the container
    container.innerHTML = '';

    // Create sections for each contract
    data.forEach((contractData, index) => {
        // Check if contractData is null or undefined
        if (!contractData) {
            return; // Skip this iteration if contractData is null or undefined
        }
        const contractSection = createChartContainer(contractData.contractName, index);
        container.appendChild(contractSection);

        // Create chart and update summary
        const chart = createChart(contractData, index);
        if (chart) {
            charts.push(chart);
            updateSummary(contractData, index);
        }
    });
}

// Function to populate project dropdown
function populateProjects() {
    // Get list of available projects from the reports data
    const projects = Object.keys(REPORTS_DATA);
    
    const select = document.getElementById('projectSelect');
    select.innerHTML = '<option value="">Select a project...</option>';
    
    projects.forEach(project => {
        const option = document.createElement('option');
        option.value = project;
        option.textContent = project;
        select.appendChild(option);
    });
}

// Event listener for project selection
document.getElementById('projectSelect').addEventListener('change', (event) => {
    const projectName = event.target.value;
    if (projectName) {
        showLoading();
        const reports = readGasReports(projectName);
        if (reports) {
            const data = calculateGasDiff(reports.solcData, reports.solxData);
            updateChart(data);
        }
        hideLoading();
    }
});

// Initialize the dashboard
document.addEventListener('DOMContentLoaded', () => {
    populateProjects();
});

// Restore calculateGasDiff function
function calculateGasDiff(solcData, solxData) {
    if (!solcData || !solxData) {
        showError('Invalid compiler data provided');
        return [];
    }

    // Group contracts by their name, filtering out objects without contract property
    const contractGroups = {};
    
    [...solcData, ...solxData].forEach(item => {
        if (!item.contract) return; // Skip items without contract property
        
        if (!contractGroups[item.contract]) {
            contractGroups[item.contract] = [];
        }
        // Add all items from the same compiler type to the group
        const compilerType = item.compiler.type;
        const settingsObject = [...solcData, ...solxData].find(
            s => s.compilerSettings && !s.contract && s.compiler.type === compilerType
        );
        
        contractGroups[item.contract].push(item);
        if (settingsObject) {
            contractGroups[item.contract].push(settingsObject);
        }
    });

    // Process each contract group
    return Object.values(contractGroups).map(contractData => 
        calculateContractDiff(contractData)
    );
}

// Function to update summary for a single contract
function updateSummary(data, index) {
    const summary = document.getElementById(`summary${index}`);
    // Use data.results, data.deployment, data.methodCount, data.contractObjects
    const { results, deployment, methodCount, contractName, contractObjects } = data;

    // Improved deployment HTML with % diff vs base
    const baseGas = deployment[0].gas;
    const deploymentHtml = deployment.map((version, idx) => {
        let percentageDiff = '';
        if (idx > 0) {
            const diff = ((version.gas - baseGas) / baseGas) * 100;
            percentageDiff = ` <span class="${diff > 0 ? 'text-red-500' : 'text-green-600'} font-medium">(${diff > 0 ? '+' : ''}${diff.toFixed(2)}%)</span>`;
        }
        // Add special background for solc-opt base row
        const rowBgClass = idx === 0 ? 'bg-blue-50 border border-blue-100' : 'bg-gray-50';
        return `
            <div class="flex justify-between items-center ${rowBgClass} rounded-lg px-4 py-3 font-mono text-sm">
                <span class="${version.compiler.type === 'solx' ? 'text-solx font-medium' : 'text-gray-700'}">${version.compiler.type} ${version.compiler.version}</span>
                <div class="flex items-center gap-2">
                    <span class="text-gray-900 font-medium">${version.gas.toLocaleString()} gas</span>
                    ${percentageDiff}
                </div>
            </div>
        `;
    }).join('');

    // Group versions by compiler type
    const versions = {
        solx: contractObjects.filter(v => v.compiler.type === 'solx'),
        solc: contractObjects.filter(v => v.compiler.type === 'solc')
    };

    // Track selected versions as state variables scoped to updateSummary
    let selectedSolxVersion = versions.solx.length > 0 ? versions.solx[0].compiler.version : '';
    let selectedSolcVersion = versions.solc.length > 0 ? versions.solc[0].compiler.version : '';

    // Function to generate all method rows for the table
    function generateMethodRows(solxVersion, solcVersion) {
        return results.map(method => {
            // Only show the function name (strip params)
            const shortName = method.name.split('(')[0];
            const solxData = method.versions.find(v => v.compiler.type === 'solx' && v.compiler.version === solxVersion);
            const solcData = method.versions.find(v => v.compiler.type === 'solc' && v.compiler.version === solcVersion);
            const solxGas = solxData ? solxData.gas : '-';
            const solcGas = solcData ? solcData.gas : '-';
            let percentDiff = '-';
            if (solxData && solcData && solxGas !== '-' && solcGas !== '-') {
                percentDiff = ((solxGas - solcGas) / solcGas * 100).toFixed(2);
            }
            return `
                <tr>
                    <td class="px-4 py-2 font-mono text-sm text-gray-900">${shortName}</td>
                    <td class="px-4 py-2 font-mono text-sm text-blue-700 text-right">${solxGas !== '-' ? solxGas.toLocaleString() : '-'}</td>
                    <td class="px-4 py-2 font-mono text-sm text-purple-700 text-right">${solcGas !== '-' ? solcGas.toLocaleString() : '-'}</td>
                    <td class="px-4 py-2 font-mono text-sm text-right ${percentDiff !== '-' && percentDiff < 0 ? 'text-green-600' : percentDiff !== '-' && percentDiff > 0 ? 'text-red-600' : 'text-gray-500'}">
                        ${percentDiff !== '-' ? percentDiff + '%' : '-'}
                    </td>
                </tr>
            `;
        }).join('');
    }

    // Function to count methods where solx is more efficient
    function countSolxEfficient(solxVersion, solcVersion) {
        return results.filter(method => {
            const solxData = method.versions.find(v => v.compiler.type === 'solx' && v.compiler.version === solxVersion);
            const solcData = method.versions.find(v => v.compiler.type === 'solc' && v.compiler.version === solcVersion);
            return solxData && solcData && solxData.gas < solcData.gas;
        }).length;
    }

    // Function to update the methods table
    function updateMethodsTable() {
        const container = summary.querySelector(`#methodsTableContainer${index}`);
        if (!container) return;

        // Count methods where solx is more efficient
        const efficientCount = countSolxEfficient(selectedSolxVersion, selectedSolcVersion);

        // Render dropdowns with correct selected value, including compiler name in each option
        const solxDropdown = `
            <select class="version-select px-2 py-1 rounded border border-gray-300 text-xs w-full" data-compiler="solx">
                ${versions.solx.map(v => `<option value="${v.compiler.version}"${v.compiler.version === selectedSolxVersion ? ' selected' : ''}>solx ${v.compiler.version}</option>`).join('')}
            </select>
        `;
        const solcDropdown = `
            <select class="version-select px-2 py-1 rounded border border-gray-300 text-xs w-full" data-compiler="solc">
                ${versions.solc.map(v => `<option value="${v.compiler.version}"${v.compiler.version === selectedSolcVersion ? ' selected' : ''}>solc ${v.compiler.version}</option>`).join('')}
            </select>
        `;

        // Render or update the summary stats with improved design
        let statsDiv = summary.querySelector('.methods-summary-stats');
        if (!statsDiv) {
            statsDiv = document.createElement('div');
            statsDiv.className = 'methods-summary-stats flex flex-wrap gap-4 items-center bg-gray-50 rounded-lg px-4 py-2 mb-6 border border-gray-200';
            container.insertAdjacentElement('beforebegin', statsDiv);
        }
        statsDiv.innerHTML = `
            <div class="flex items-center gap-2 text-gray-700 text-sm">
                <span>Total Methods Tested</span>
                <span class="inline-block bg-blue-100 text-blue-800 text-xs font-semibold px-2 py-0.5 rounded-full">${methodCount}</span>
            </div>
            <div class="flex items-center gap-2 text-gray-700 text-sm">
                <span>Methods where solx is more efficient</span>
                <span class="inline-block bg-green-100 text-green-800 text-xs font-semibold px-2 py-0.5 rounded-full">${efficientCount}</span>
            </div>
        `;

        const tableHtml = `
            <div class="overflow-x-auto">
                <table class="min-w-full border border-gray-200 rounded-xl overflow-hidden">
                    <thead class="bg-gray-50 rounded-t-xl">
                        <tr>
                            <th class="px-4 py-2 text-left text-xs font-semibold text-gray-700 rounded-tl-xl">Method</th>
                            <th class="px-4 py-2 text-right text-xs font-semibold text-blue-700">${solxDropdown}</th>
                            <th class="px-4 py-2 text-right text-xs font-semibold text-purple-700">${solcDropdown}</th>
                            <th class="px-4 py-2 text-right text-xs font-semibold text-gray-700 rounded-tr-xl">% Diff (solx vs solc)</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${generateMethodRows(selectedSolxVersion, selectedSolcVersion)}
                    </tbody>
                </table>
            </div>
        `;
        container.innerHTML = tableHtml;

        // Add event listeners to dropdowns in the table header
        container.querySelectorAll('.version-select').forEach(select => {
            select.addEventListener('change', (e) => {
                if (select.dataset.compiler === 'solx') {
                    selectedSolxVersion = select.value;
                } else {
                    selectedSolcVersion = select.value;
                }
                updateMethodsTable();
            });
        });
    }

    // Always render the summary sections first
    summary.innerHTML = `
        <div class="space-y-6">
            <div class="bg-white rounded-xl shadow-sm overflow-hidden">
                <div class="border-b border-gray-200">
                    <h4 class="text-lg font-semibold text-gray-900 px-6 py-4">Contract Deployment</h4>
                </div>
                <div class="p-6 space-y-3">
                    ${deploymentHtml}
                </div>
            </div>

            <div class="bg-white rounded-xl shadow-sm overflow-hidden">
                <div class="border-b border-gray-200">
                    <h4 class="text-lg font-semibold text-gray-900 px-6 py-4">Methods Summary</h4>
                </div>
                <div class="p-6">
                    <div id="methodsTableContainer${index}"></div>
                    <div id="methodsDebug${index}" class="text-red-600 mt-4"></div>
                </div>
            </div>
        </div>
    `;

    // Initial render
    updateMethodsTable();
}

// Restore createChartContainer and createChart functions
function createChartContainer(contractName, index) {
    const container = document.createElement('div');
    container.className = 'contract-section';
    container.innerHTML = `
        <h3 class="text-lg font-semibold text-gray-900 mb-2">${contractName}</h3>
        <div class="chart-container mb-6">
            <canvas id="gasChart${index}"></canvas>
        </div>
        <div class="summary" id="summary${index}"></div>
    `;
    return container;
}

function createChart(data, index) {
    const ctx = document.getElementById(`gasChart${index}`).getContext('2d');
    // Sort results by maximum gas usage
    const sortedResults = [...data.results].sort((a, b) => {
        const maxA = Math.max(...a.versions.map(v => Number(v.gas)));
        const maxB = Math.max(...b.versions.map(v => Number(v.gas)));
        return maxB - maxA;
    });
    // Calculate dynamic height based on number of methods
    const baseHeight = 400;
    const heightPerMethod = 40;
    const minMethods = 10;
    const methodCount = sortedResults.length;
    const dynamicHeight = methodCount <= minMethods ? baseHeight : baseHeight + (methodCount - minMethods) * heightPerMethod;
    // Set container height
    const container = ctx.canvas.parentElement;
    container.style.height = `${dynamicHeight}px`;
    // Create datasets for each compiler version
    const datasets = [];
    const allVersions = data.results[0].versions;
    allVersions.forEach((version, vIndex) => {
        const compilerType = version.compiler.type;
        const colorScheme = colorSchemes[compilerType];
        const backgroundColor = colorScheme.getColor(vIndex, allVersions.length);
        // Create dataset for this version
        const dataset = {
            label: `${compilerType} ${version.compiler.version}`,
            backgroundColor,
            borderColor: colorScheme.getBorderColor(backgroundColor),
            borderWidth: 1,
            barPercentage: 0.95,
            categoryPercentage: 0.8,
            data: []
        };
        // Fill in data for each method in sorted order
        sortedResults.forEach(method => {
            const matchingVersion = method.versions.find(v => v.compiler.type === version.compiler.type && v.compiler.version === version.compiler.version);
            dataset.data.push(matchingVersion ? Number(matchingVersion.gas) : 0);
        });
        datasets.push(dataset);
    });
    // Create chart
    return new Chart(ctx, {
        type: 'bar',
        data: {
            labels: sortedResults.map(r => r.name),
            datasets
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            indexAxis: 'y',
            plugins: {
                title: {
                    display: true,
                    text: 'Gas Usage Comparison by Method',
                    font: { size: 14 }
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            const methodName = sortedResults[context.dataIndex].name;
                            const version = allVersions[context.datasetIndex];
                            const matchingVersion = sortedResults[context.dataIndex].versions.find(v => v.compiler.type === version.compiler.type && v.compiler.version === version.compiler.version);
                            const gas = matchingVersion ? Number(matchingVersion.gas).toLocaleString() : '0';
                            return `${version.compiler.type} ${version.compiler.version}: ${gas} gas`;
                        }
                    }
                }
            },
            scales: {
                y: {
                    beginAtZero: false,
                    grid: { display: false }
                },
                x: {
                    type: 'linear',
                    beginAtZero: true,
                    grid: { color: '#e0e0e0' },
                    title: {
                        display: true,
                        text: 'Gas Usage',
                        font: { size: 12 }
                    },
                    ticks: {
                        callback: function(value) {
                            return Number(value).toLocaleString();
                        }
                    }
                }
            }
        }
    });
} 
