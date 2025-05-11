# E2E Test Runner

E2E testing tool using YAML configurations and WebDriver.

## Commands

- run Executes E2E test scenarios.
- config Prints parsed `e2e.yaml` or a section.

### Global Options

- `-f, --file <FILE>`: Path to config file (default: `e2e.yaml`).

## Command Examples

### Running Scenarios

- Run all scenarios from `e2e.yaml`:
  ```bash
  e2e run
  ```
- Run specific scenarios (`login_success`, `search_product`):
  ```bash
  e2e run login_success search_product
  ```
- Run all scenarios from `my_tests.yaml`:
  ```bash
  e2e -f my_tests.yaml run
  ```
- Run a specific scenario from `my_tests.yaml`:
  ```bash
  e2e --file my_tests.yaml run login_success
  ```

### Inspecting Configuration

- Print entire config from `e2e.yaml`:
  ```bash
  e2e config
  ```
- Print `driver` section from `e2e.yaml`:
  ```bash
  e2e config driver
  ```
- Print `scenarios` section from `my_tests.yaml`:
  ```bash
  e2e -f my_tests.yaml config scenarios
  ```

## `e2e.yaml` Syntax

`e2e.yaml` defines E2E test behavior with these main sections:

| Key         | Required | Description                                                      |
|-------------|----------|------------------------------------------------------------------|
| `driver`    | Yes      | WebDriver configuration.                                         |
| `vars`      | No       | Variables for scenarios and tasks.                               |
| `tasks`     | No       | Reusable tasks composed of steps.                                |
| `scenarios` | Yes      | Test scenarios with names and steps.                             |

### `driver` (Required)

WebDriver configuration.

| Key        | Type    | Description                                                 |
|------------|---------|-------------------------------------------------------------|
| `host`     | String  | WebDriver server hostname (e.g., `localhost`).              |
| `port`     | String  | WebDriver server port (e.g., `4444`).                     |
| `headless` | Boolean | Run browser in headless mode (`true`/`false`).            |
| `window`   | Object  | Browser window dimensions.                                  |
| `window.x` | Integer | Window x-coordinate.                                        |
| `window.y` | Integer | Window y-coordinate.                                        |
| `window.width` | Integer | Window width (pixels).                                      |
| `window.height`| Integer | Window height (pixels).                                     |


**Example:**
```yaml
driver:
  host: localhost
  port: 4444
  headless: true
  window:
    x: 0
    y: 0
    width: 1920
    height: 1080
```

### `vars` (Optional)

Variables for scenario steps and task arguments. Use `{variable_name}`.

**Example:**
```yaml
vars:
  username: user
  password: passw0rd
```

### `tasks` (Optional)

Reusable tasks with optional arguments (`arg_names`) and steps.

| Key         | Type             | Description                                                       |
|-------------|------------------|-------------------------------------------------------------------|
| `arg_names` | List of Strings  | (Optional) Argument names for the task.                           |
| `steps`     | List of Steps    | Actions for this task. See `steps`.                             |

**Example:**
```yaml
tasks:
  login:
    arg_names:
      - username
      - password
    steps:
      - !send_keys { selector: "#username", value: "{username}" }
      - !send_keys { selector: "#password", value: "{password}" }
      - !click { selector: "#login-button" }
```

### `scenarios` (Required)

Test scenarios, each with an ID, `name`, and `steps`.

| Key     | Type          | Description                                         |
|---------|---------------|-----------------------------------------------------|
| `name`  | String        | Scenario name.                                      |
| `steps` | List of Steps | Actions for this scenario. See `steps`.           |


**Example:**
```yaml
scenarios:
  login_success:
    name: "Successful Login"
    steps:
      - !goto "{baseUrl}/login"
      - !task_run { id: login, args: ["{username}", "securepassword"] }
      - !wait_displayed { selector: "#dashboard", timeout: 5000, interval: 500 }
      - !assert_eq { kind: text, expected: "Welcome, {username}!", selector: ".welcome-message" }
```

### `steps` (Used in `tasks` and `scenarios`)

Steps are actions specified with YAML tags.

- `!goto <URL>`: Navigates browser to URL.
  ```yaml
  - !goto "http://example.com"
  ```
- `!click <SELECTOR>`: Clicks element by CSS selector.
  ```yaml
  - !click "#submit-button"
  ```
- `!focus <SELECTOR>`: Focuses element by CSS selector.
  ```yaml
  - !focus "input[name='email']"
  ```
- `!send_keys { selector: <SELECTOR>, value: <TEXT> }`: Clears and types text into element by CSS selector.
  ```yaml
  - !send_keys { selector: "#search-box", value: "Hello, World!" }
  ```
- `!screen_shot <FILE_PATH>`: Takes screenshot, saves to path. Creates dirs if needed.
  ```yaml
  - !screen_shot "reports/screenshots/homepage.png"
  ```
- `!wait_displayed { selector: <SELECTOR>, timeout: <MILLISECONDS>, interval: <MILLISECONDS> }`: Waits for element by CSS selector to be displayed.
  - `timeout`: Max wait time (ms).
  - `interval`: Check interval (ms).
  ```yaml
  - !wait_displayed { selector: ".loading-spinner", timeout: 10000, interval: 500 }
  ```
- `!accept_alert`: Accepts current browser alert.
  ```yaml
  - !accept_alert
  ```
- `!task_run { id: <TASK_ID>, args: [ARG1, ARG2, ...] }`: Runs a predefined task.
  - `id`: Task ID (from `tasks` section).
  - `args`: (Optional) Task arguments. Replaces placeholders in `arg_names`.
  ```yaml
  - !task_run { id: login, args: ["myuser", "mypass"] }
  ```
- `!assert_eq { kind: <VALUE_KIND>, expected: <EXPECTED_VALUE>, selector: <SELECTOR> }`: Asserts element property matches expected value.
  - `kind`: Value type to check:
    - `text`: Element's inner text.
    - `id`: Element's `id` attribute.
    - `class`: Element's `class` attribute.
  - `expected`: Expected value.
  - `selector`: Element's CSS selector.
  ```yaml
  - !assert_eq { kind: text, expected: "Login Successful", selector: "h1.title" }
  ```

### Variable Expansion

Variables from `vars` or task arguments can be used in step strings (URLs, selectors, text, paths) with `{variable_name}`. Escape literal braces: `{{`, `}}`.
```yaml
vars:
  domain: "example.com"
scenarios:
  test_search:
    name: "Test Search on Domain"
    steps:
      - !goto "http://{domain}"
      - !send_keys { selector: "#search", value: "testing on {{domain}}" } # literal '{{domain}}'
```
