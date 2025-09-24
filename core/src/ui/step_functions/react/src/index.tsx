import React from 'react';
import ReactDOM from 'react-dom/client';
import WorkflowGraph from './WorkflowGraph';

// Global function to initialize the React component
(window as any).initStepFunctionsGraph = (containerId: string) => {
  const container = document.getElementById(containerId);
  if (container) {
    const root = ReactDOM.createRoot(container);
    root.render(<WorkflowGraph />);
  }
};

// Auto-initialize if container exists
document.addEventListener('DOMContentLoaded', () => {
  const container = document.getElementById('stepfunctions-graph');
  if (container) {
    (window as any).initStepFunctionsGraph('stepfunctions-graph');
  }
});
