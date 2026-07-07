Feature: Validate command CLI surface
  As a user
  I want to invoke the validate command with various arguments
  So that I can validate Markdown documents against schemas

  Background:
    Given a temporary working directory

  Scenario: Missing required --document argument
    When I run mdast-check validate --schema schema.json
    Then the exit code should be 2
    And stderr should contain "missing required argument: --document"

  Scenario: Missing required --schema argument
    When I run mdast-check validate --document doc.md
    Then the exit code should be 2
    And stderr should contain "missing required argument: --schema"

  Scenario: Valid invocation with both required arguments
    Given a file named "doc.md" with content:
      """
      # Test Document
      """
    And a file named "schema.json" with content:
      """
      {}
      """
    When I run mdast-check validate --document doc.md --schema schema.json
    Then the exit code should be 0
    And stdout should contain "Validating"

  Scenario: Quiet mode suppresses output
    Given a file named "doc.md" with content:
      """
      # Test Document
      """
    And a file named "schema.json" with content:
      """
      {}
      """
    When I run mdast-check validate --document doc.md --schema schema.json --quiet
    Then the exit code should be 0
    And stdout should be empty

  Scenario: Help flag displays usage
    When I run mdast-check validate --help
    Then the exit code should be 0
    And stdout should contain "--document"
    And stdout should contain "--schema"

  Scenario: Version flag displays version
    When I run mdast-check --version
    Then the exit code should be 0
