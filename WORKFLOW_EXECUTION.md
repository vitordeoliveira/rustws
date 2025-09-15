# Workflow Execution Flow

## Overview

Step Functions execute workflows by chaining lambda functions together. Each lambda's output becomes the next lambda's input.

## Execution Steps

### 1. Load Workflow

```
workflow_name → load JSON file → parse states
```

### 2. Execute States Sequentially

```
input_data → lambda1 → output1 → lambda2 → output2 → lambda3 → final_output
```

### 3. State Types

- **Task**: Execute lambda function (`rustws:lambda:function_name`)
- **Succeed**: Workflow completed successfully
- **Fail**: Workflow failed with error
- **Choice/Wait**: Not implemented yet

### 4. Data Flow

```json
// Input to first lambda
{"message": "Hello", "count": 5}

// Output from lambda1 becomes input to lambda2
{"text": "Processed: Hello (original count: 5)", "count": 15}

// Final workflow output
{"text": "Processed: Processed: Hello (original count: 5) (original count: 15)", "count": 25}
```

## Example Workflow

```json
{
  "Comment": "Chain two hello_world lambdas",
  "StartAt": "First",
  "States": {
    "First": {
      "Type": "Task",
      "Resource": "rustws:lambda:hello_world",
      "Next": "Second"
    },
    "Second": {
      "Type": "Task",
      "Resource": "rustws:lambda:hello_world"
    }
  }
}
```

## Implementation

- Uses shared wasmer engine/store for lambda execution
- Each lambda gets JSON input, returns JSON output
- Sequential execution until terminal state
- Returns execution time, states executed, and final output

