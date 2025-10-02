use uuid::Uuid;

use crate::domain::authentication::{
    entities::{AuthSession, AuthSessionParams, AuthenticationError},
    ports::{AuthSessionRepository, AuthSessionService},
    value_objects::CreateAuthSessionRequest,
};

#[derive(Clone)]
pub struct AuthSessionServiceImpl<R: AuthSessionRepository> {
    pub repository: R,
}

impl<R: AuthSessionRepository> AuthSessionServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: AuthSessionRepository> AuthSessionService for AuthSessionServiceImpl<R> {
    async fn create_session(
        &self,
        request: CreateAuthSessionRequest,
    ) -> Result<AuthSession, AuthenticationError> {
        let params = AuthSessionParams {
            realm_id: request.realm_id,
            client_id: request.client_id,
            redirect_uri: request.redirect_uri,
            response_type: request.response_type,
            scope: request.scope,
            state: request.state,
            nonce: request.nonce,
            user_id: request.user_id,
            code: None,
            authenticated: false,
        };
        let session = AuthSession::new(params);
        self.repository.create(&session).await?;
        Ok(session)
    }

    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        self.repository.get_by_session_code(session_code).await
    }

    async fn get_by_code(&self, code: String) -> Result<AuthSession, AuthenticationError> {
        self.repository
            .get_by_code(code)
            .await?
            .ok_or(AuthenticationError::NotFound)
    }

    async fn update_code(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        self.repository
            .update_code_and_user_id(session_code, code, user_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::ports::test::get_mock_auth_session_repository_with_clone_expectations;
    use std::future::ready;

    fn build_request() -> CreateAuthSessionRequest {
        CreateAuthSessionRequest {
            realm_id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            redirect_uri: "https://example.com/callback".to_string(),
            response_type: "code".to_string(),
            scope: "openid".to_string(),
            state: Some("state123".to_string()),
            nonce: None,
            user_id: None,
        }
    }

    #[tokio::test]
    async fn create_session_persists_and_returns_session() {
        // Arrange
        let mut repo = get_mock_auth_session_repository_with_clone_expectations();
        repo.expect_create()
            .returning(|s: &AuthSession| Box::pin(ready(Ok(s.clone()))));
        let service = AuthSessionServiceImpl::new(repo);
        let req = build_request();

        // Act
        let out = service.create_session(req.clone()).await.unwrap();

        // Assert
        assert_eq!(out.redirect_uri, req.redirect_uri);
        assert_eq!(out.response_type, req.response_type);
        assert_eq!(out.scope, req.scope);
        assert_eq!(out.state, req.state);
        assert_eq!(out.user_id, None);
        assert_eq!(out.code, None);
        assert!(!out.authenticated);
        assert_eq!(out.realm_id, req.realm_id);
        assert_eq!(out.client_id, req.client_id);
    }

    #[tokio::test]
    async fn get_by_session_code_returns_session() {
        // Arrange
        let mut repo = get_mock_auth_session_repository_with_clone_expectations();
        let req = build_request();
        let params = AuthSessionParams {
            realm_id: req.realm_id,
            client_id: req.client_id,
            redirect_uri: req.redirect_uri.clone(),
            response_type: req.response_type.clone(),
            scope: req.scope.clone(),
            state: req.state.clone(),
            nonce: req.nonce.clone(),
            user_id: req.user_id,
            code: None,
            authenticated: false,
        };
        let session = AuthSession::new(params);
        let expected_code = Uuid::new_v4();
        let session_clone = session.clone();
        repo.expect_get_by_session_code()
            .returning(move |sc: Uuid| {
                assert_eq!(sc, expected_code);
                Box::pin(ready(Ok(session_clone.clone())))
            });
        let service = AuthSessionServiceImpl::new(repo);

        // Act
        let out = service.get_by_session_code(expected_code).await.unwrap();

        // Assert
        assert_eq!(out.id, session.id);
        assert_eq!(out.client_id, session.client_id);
        assert_eq!(out.realm_id, session.realm_id);
    }

    #[tokio::test]
    async fn get_by_code_none_maps_to_not_found() {
        // Arrange
        let mut repo = get_mock_auth_session_repository_with_clone_expectations();
        repo.expect_get_by_code()
            .returning(|code: String| {
                assert_eq!(code, "abc");
                Box::pin(ready(Ok(None)))
            });
        let service = AuthSessionServiceImpl::new(repo);

        // Act
        let err = service.get_by_code("abc".to_string()).await.err().unwrap();

        // Assert
        assert!(matches!(err, AuthenticationError::NotFound));
    }

    #[tokio::test]
    async fn update_code_returns_updated_session() {
        // Arrange
        let mut repo = get_mock_auth_session_repository_with_clone_expectations();
        let session_code = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let new_code = "newcode".to_string();
        let new_code_for_assert = new_code.clone();

        repo.expect_update_code_and_user_id()
            .returning(move |sc: Uuid, code: String, uid: Uuid| {
                assert_eq!(sc, session_code);
                assert_eq!(code, new_code_for_assert);
                assert_eq!(uid, user_id);
                let params = AuthSessionParams {
                    realm_id: Uuid::new_v4(),
                    client_id: Uuid::new_v4(),
                    redirect_uri: "https://cb".into(),
                    response_type: "code".into(),
                    scope: "openid".into(),
                    state: None,
                    nonce: None,
                    user_id: Some(uid),
                    code: Some(code.clone()),
                    authenticated: true,
                };
                let session = AuthSession::new(params);
                Box::pin(ready(Ok(session)))
            });
        let service = AuthSessionServiceImpl::new(repo);

        // Act
        let out = service
            .update_code(session_code, new_code.clone(), user_id)
            .await
            .unwrap();

        // Assert
        assert_eq!(out.user_id, Some(user_id));
        assert_eq!(out.code, Some(new_code));
        assert!(out.authenticated);
    }
}
