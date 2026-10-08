Feature: RP-initiated logout
  An application ends the user's session and sends the browser back to an
  address it registered for that purpose.

  Background:
    Given a confidential application "web"
    And "web" accepts "https://app.example/logged-out" after logout
    And a registered user "alice"

  Scenario: Logout with a fresh ID token returns to the registered address
    Given "alice" has signed in to "web"
    When "alice" logs out of "web" towards "https://app.example/logged-out" with state "bye"
    Then the browser is sent to "https://app.example/logged-out" with state "bye"

  Scenario: Logout towards an unregistered address is refused
    Given "alice" has signed in to "web"
    When "alice" logs out of "web" towards "https://evil.example/" with state "bye"
    Then the browser is not sent to "evil.example"

  @wip @SSO-05
  Scenario: Logout with an expired ID token still returns to the registered address
    Given "web" issues ID tokens that are already expired
    And "alice" has signed in to "web"
    When "alice" logs out of "web" towards "https://app.example/logged-out" with state "bye"
    Then the browser is sent to "https://app.example/logged-out" with state "bye"
