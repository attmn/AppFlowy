use crate::{entities::workspace::*, errors::*, module::WorkspaceUser, sql_tables::workspace::*};
use flowy_database::{prelude::*, schema::workspace_table};

use flowy_database::schema::workspace_table::dsl;
use flowy_dispatch::prelude::DispatchFuture;
use std::sync::Arc;

pub struct WorkspaceController {
    pub user: Arc<dyn WorkspaceUser>,
}

impl WorkspaceController {
    pub fn new(user: Arc<dyn WorkspaceUser>) -> Self { Self { user } }

    pub async fn save_workspace(
        &self,
        params: CreateWorkspaceParams,
    ) -> Result<WorkspaceDetail, WorkspaceError> {
        let workspace = Workspace::new(params);
        let detail: WorkspaceDetail = workspace.clone().into();

        let _ = diesel::insert_into(workspace_table::table)
            .values(workspace)
            .execute(&*(self.user.db_connection()?))?;

        let _ = self.user.set_cur_workspace_id(&detail.id).await?;

        Ok(detail)
    }

    pub fn get_workspace(
        &self,
        workspace_id: &str,
    ) -> DispatchFuture<Result<Workspace, WorkspaceError>> {
        let user = self.user.clone();
        let workspace_id = workspace_id.to_owned();
        DispatchFuture {
            fut: Box::pin(async move {
                let workspace = dsl::workspace_table
                    .filter(workspace_table::id.eq(&workspace_id))
                    .first::<Workspace>(&*(user.db_connection()?))?;

                // TODO: fetch workspace from remote server
                Ok(workspace)
            }),
        }
    }

    pub fn update_workspace(&self, params: UpdateWorkspaceParams) -> Result<(), WorkspaceError> {
        let changeset = WorkspaceChangeset::new(params);
        let conn = self.user.db_connection()?;
        diesel_update_table!(workspace_table, changeset, conn);

        Ok(())
    }

    pub async fn get_user_workspace_detail(&self) -> Result<UserWorkspaceDetail, WorkspaceError> {
        let user_workspace = self.user.get_cur_workspace().await?;
        let workspace = self.get_workspace(&user_workspace.workspace_id).await?;

        Ok(UserWorkspaceDetail {
            owner: user_workspace.owner,
            workspace: workspace.into(),
        })
    }
}
