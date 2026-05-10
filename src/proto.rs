// Temporarily mocked protobuf types for compilation

// These modules will be properly generated from proto files when we re-enable the workspace
pub mod postdevai {
    // Mock types needed for the dragon_node_service.rs
    
    // Search similar response
    pub mod search_similar_response {
        use uuid::Uuid;
        
        #[derive(Debug, Clone)]
        pub struct Result {
            pub id: Option<super::Uuid>,
            pub score: f32,
        }
    }
    
    // Get related response
    pub mod get_related_response {
        use uuid::Uuid;
        
        #[derive(Debug, Clone)]
        pub struct Relation {
            pub source_id: Option<super::Uuid>,
            pub relation: String,
            pub target_id: Option<super::Uuid>,
        }
    }
    
    // UUID wrapper
    #[derive(Debug, Clone)]
    pub struct Uuid {
        pub value: String,
    }
    
    // Empty mock implementations for the services
    pub mod dragon_node_service_server {
        use tonic::{Request, Response, Status};
        
        #[tonic::async_trait]
        pub trait DragonNodeService {}
        
        pub struct DragonNodeServiceServer<T>(pub T);
    }
}

// Re-export mocked types
pub use postdevai::{
    dragon_node_service_server::DragonNodeService,
    search_similar_response, get_related_response
};