# Step Functions React Flow Integration

This directory contains the React Flow integration for the Step Functions create page, providing visual workflow design capabilities.

## Structure

```
react/
├── src/
│   ├── index.tsx          # React app entry point
│   └── WorkflowGraph.tsx  # Main React Flow component
├── dist/
│   └── bundle.js          # Compiled React bundle
├── package.json           # React dependencies
├── webpack.config.js      # Webpack build configuration
└── tsconfig.json         # TypeScript configuration
```

## Features

- **Visual Workflow Designer**: Interactive React Flow graph for designing Step Functions workflows
- **Real-time Sync**: Changes in the visual graph automatically update the JSON textarea
- **Node Types**: Support for Task, Choice, Wait, Succeed, Fail, and Start nodes
- **Drag & Drop**: Intuitive node positioning and connection
- **JSON Integration**: Seamless integration with existing form validation

## Development

### Prerequisites

- Node.js and npm
- React 18
- ReactFlow 11

### Building

```bash
# Install dependencies
cd src/ui/step_functions/react
npm install

# Build for production
npm run build

# Development build with watch
npm run dev
```

### Integration

The React component is automatically loaded when the Step Functions create page is accessed. The component:

1. Renders in the `#stepfunctions-graph` container
2. Syncs with the `#workflowDefinition` textarea
3. Parses JSON workflow definitions into visual nodes
4. Updates the JSON when nodes/edges are modified

## Usage

1. Navigate to `/step-functions/create`
2. The visual workflow designer appears above the JSON textarea
3. Drag nodes to design your workflow
4. Connect nodes by dragging from one to another
5. Changes automatically sync to the JSON textarea below
6. Use "Load Template" to populate with sample workflow
7. Use "Validate JSON" to check syntax

## Node Types

- **Start** (Green): Entry point of the workflow
- **Task** (Blue): Lambda function or service call
- **Choice** (Yellow): Conditional branching
- **Wait** (Purple): Delay or pause
- **Succeed** (Green): Successful completion
- **Fail** (Red): Error termination

## Technical Details

- **Bundle Size**: ~156KB minified
- **External Dependencies**: React and ReactDOM loaded from CDN
- **Build Tool**: Webpack 5 with TypeScript support
- **Styling**: Tailwind CSS classes for consistent design
- **State Management**: React hooks for local state

## Troubleshooting

If the React Flow component doesn't load:

1. Check browser console for errors
2. Ensure the bundle was built: `npm run build`
3. Verify the route `/step-functions/react/bundle.js` is accessible
4. Check that React and ReactDOM CDN links are loading

The fallback JavaScript will log helpful error messages if the bundle is missing.
