Feature: Authorization code flow
  An application signs a user in through the authorization endpoint and
  redeems the code it receives at the token endpoint.

  Background:
    Given a confidential application "web"
    And a registered user "alice"

  Scenario: A user signs in and the application receives an ID token
    When "alice" signs in to "web"
    Then "web" receives an ID token for "alice"

  Scenario: A redeemed code cannot be redeemed again
    Given "alice" has signed in to "web"
    When "web" redeems the same code again
    Then the token endpoint refuses with "invalid_grant"

  Scenario: An application without PKCE requirement can still use S256
    Given a public application "spa"
    When "alice" signs in to "spa" with a PKCE S256 challenge
    Then "spa" receives an ID token for "alice"

  @SSO-01
  Scenario: Redeeming a code twice revokes the tokens it already produced
    Given "alice" has signed in to "web"
    When "web" redeems the same code again
    Then the access token first issued to "web" is no longer active

  @SSO-04
  Scenario: A malformed prompt never redirects to an unregistered address
    When an authorization request for "web" carries prompt "none login" and redirect_uri "https://evil.example/cb"
    Then the browser is not sent to "evil.example"

  @wip @SSO-10
  Scenario: An unsupported response type is refused
    When an authorization request for "web" asks for response_type "token"
    Then the authorization request is refused with "unsupported_response_type"

  @wip @SSO-20
  Scenario: A public application must use PKCE
    Given a public application "spa"
    When an authorization request for "spa" is sent without PKCE
    Then the authorization request is refused with "invalid_request"
