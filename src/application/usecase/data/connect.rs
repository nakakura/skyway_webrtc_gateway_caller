use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;

use crate::application::dto::Parameter;
use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::data::entity::{ConnectQuery, DataConnection, DataConnectionIdWrapper};
use crate::domain::webrtc::data::repository::DataRepository;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct ConnectService {
    #[shaku(inject)]
    api: Arc<dyn DataRepository>,
}

#[async_trait]
impl Service for ConnectService {
    async fn execute(&self, params: Parameter) -> Result<ResponseMessage, error::Error> {
        let query = params.deserialize::<ConnectQuery>()?;
        let connection = DataConnection::try_create(self.api.clone(), query).await?;
        let wrapper = DataConnectionIdWrapper {
            data_connection_id: connection.data_connection_id().clone(),
        };
        Ok(DataResponseMessageBodyEnum::Connect(wrapper).create_response_message())
    }
}

#[cfg(test)]
mod test_create_data {
    use super::*;
    use crate::di::DataConnectServiceContainer;
    use crate::domain::webrtc::data::repository::MockDataRepository;
    use crate::domain::webrtc::data::value_object::DataConnectionId;
    use crate::domain::webrtc::peer::value_object::{PeerId, Token};
    use crate::error;

    #[tokio::test]
    async fn success() {
        let data_connection_id =
            DataConnectionId::try_create("dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // 期待値を生成
        let expected = DataResponseMessageBodyEnum::Connect(DataConnectionIdWrapper {
            data_connection_id: data_connection_id.clone(),
        })
        .create_response_message();

        // CONNECTに成功する場合のMockを作成
        let mut mock = MockDataRepository::default();
        mock.expect_connect()
            .returning(move |_| Ok(data_connection_id.clone()));

        // Mockを埋め込んだServiceを生成
        let module = DataConnectServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let connect_service: Arc<dyn Service> = module.resolve();

        // 引数を生成
        let message = ConnectQuery {
            peer_id: PeerId("peer_id".into()),
            token: Token::try_create("pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap(),
            options: None,
            target_id: PeerId("target_id".into()),
            params: None,
            redirect_params: None,
        };
        let message = serde_json::to_value(message).unwrap();

        //実行
        let result = connect_service.execute(Parameter(message)).await.unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_params() {
        // このMockは呼ばれないので、初期化の必要はない
        let mock = MockDataRepository::default();

        // Mockを埋め込んだServiceを生成
        let module = DataConnectServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let connect_service: Arc<dyn Service> = module.resolve();

        // 間違った値で実行
        let result = connect_service
            .execute(Parameter(serde_json::Value::Bool(true)))
            .await;

        // 削除済みの場合エラーが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
