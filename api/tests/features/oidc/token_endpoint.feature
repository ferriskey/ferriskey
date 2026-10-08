Feature: Token endpoint client authentication and refresh
  Every grant authenticates the client that calls it, and refresh tokens
  rotate on each use.

  Background:
    Given a confidential application "web"
    And a registered user "alice"

  Scenario: A rotated refresh token cannot be used again
    Given "alice" has signed in to "web"
    And "web" has refreshed its tokens once
    When "web" refreshes with the refresh token it already rotated
    Then the token endpoint refuses with "invalid_grant"

  Scenario: A service account obtains a token with its credentials
    Given a confidential application "svc" with a service account
    When "svc" requests a token with its own credentials
    Then the token endpoint issues an access token

  @wip @SSO-02
  Scenario: A confidential application must authenticate to refresh
    Given "alice" has signed in to "web"
    When "web" refreshes its tokens without its client secret
    Then the token endpoint refuses with "invalid_client"

  @wip @SSO-02
  Scenario: A disabled application cannot refresh
    Given "alice" has signed in to "web"
    And "web" is disabled
    When "web" refreshes its tokens
    Then the token endpoint refuses with "invalid_client"

  @wip @SSO-03
  Scenario: A disabled application cannot use its service account
    Given a confidential application "svc" with a service account
    And "svc" is disabled
    When "svc" requests a token with its own credentials
    Then the token endpoint refuses with "invalid_client"

  @wip @SSO-14
  Scenario: A rotated refresh token is reported inactive
    Given "alice" has signed in to "web"
    And "web" has refreshed its tokens once
    When "web" introspects the refresh token it already rotated
    Then the token is reported inactive
