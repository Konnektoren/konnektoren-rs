Feature: Asset loading

  Scenario: A declared asset collection loads its asset
    Given an application has a greeting asset
    When the greeting is loaded from its collection
    Then the loaded greeting should be "Hallo"

  Scenario: An application loads an asset selected at runtime
    Given an application has a greeting asset
    When the greeting is loaded by its runtime path
    Then the loaded greeting should be "Hallo"

  Scenario: Repeated asset requests share a loaded value
    Given an application has a greeting asset
    When the greeting is requested twice
    Then both requests should share the loaded greeting
