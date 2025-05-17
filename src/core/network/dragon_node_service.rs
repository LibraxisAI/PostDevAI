use std::sync::Arc;
use parking_lot::RwLock;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::core::memory::ramlake::RamLake;
use crate::mlx::models::MLXModelManager;

// Import generated code from proto
tonic::include_proto!("postdevai");

// Import generated service definition
pub use self::dragon_node_service_server::{DragonNodeService, DragonNodeServiceServer};

pub struct DragonNodeServiceImpl {
    ram_lake: Arc<RwLock<RamLake>>,
    model_manager: Arc<RwLock<MLXModelManager>>,
}

impl DragonNodeServiceImpl {
    pub fn new(ram_lake: Arc<RwLock<RamLake>>, model_manager: Arc<RwLock<MLXModelManager>>) -> Self {
        Self {
            ram_lake,
            model_manager,
        }
    }
}

#[tonic::async_trait]
impl DragonNodeService for DragonNodeServiceImpl {
    // RAM-Lake related operations
    
    async fn store_code(
        &self,
        request: Request<StoreCodeRequest>,
    ) -> Result<Response<StoreCodeResponse>, Status> {
        let req = request.into_inner();
        
        // Call RAM-Lake store_code
        let result = self.ram_lake.write().store_code(
            &req.path,
            &req.content,
            &req.language,
        ).map_err(|e| Status::internal(format!("Failed to store code: {}", e)))?;
        
        // Convert Uuid to string
        let id = Uuid {
            value: result.to_string(),
        };
        
        // Return response
        Ok(Response::new(StoreCodeResponse { id: Some(id) }))
    }
    
    async fn index_code(
        &self,
        request: Request<IndexCodeRequest>,
    ) -> Result<Response<IndexCodeResponse>, Status> {
        let req = request.into_inner();
        
        // Parse UUID
        let code_id = req.code_id.ok_or_else(|| Status::invalid_argument("Missing code_id"))?;
        let code_uuid = Uuid::parse_str(&code_id.value)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;
        
        // Call RAM-Lake index_code
        self.ram_lake.write().index_code(
            code_uuid,
            req.embeddings,
        ).map_err(|e| Status::internal(format!("Failed to index code: {}", e)))?;
        
        // Return response
        Ok(Response::new(IndexCodeResponse { success: true }))
    }
    
    async fn store_event(
        &self,
        request: Request<StoreEventRequest>,
    ) -> Result<Response<StoreEventResponse>, Status> {
        let req = request.into_inner();
        
        // Call RAM-Lake store_event
        let result = self.ram_lake.write().store_event(
            &req.event_type,
            &req.content,
        ).map_err(|e| Status::internal(format!("Failed to store event: {}", e)))?;
        
        // Convert Uuid to string
        let id = Uuid {
            value: result.to_string(),
        };
        
        // Return response
        Ok(Response::new(StoreEventResponse { id: Some(id) }))
    }
    
    async fn store_metadata(
        &self,
        request: Request<StoreMetadataRequest>,
    ) -> Result<Response<StoreMetadataResponse>, Status> {
        let req = request.into_inner();
        
        // Parse UUIDs
        let source_id = req.source_id.ok_or_else(|| Status::invalid_argument("Missing source_id"))?;
        let target_id = req.target_id.ok_or_else(|| Status::invalid_argument("Missing target_id"))?;
        
        let source_uuid = Uuid::parse_str(&source_id.value)
            .map_err(|_| Status::invalid_argument("Invalid source UUID format"))?;
        let target_uuid = Uuid::parse_str(&target_id.value)
            .map_err(|_| Status::invalid_argument("Invalid target UUID format"))?;
        
        // Call RAM-Lake store_metadata
        self.ram_lake.write().store_metadata(
            source_uuid,
            &req.relation,
            target_uuid,
        ).map_err(|e| Status::internal(format!("Failed to store metadata: {}", e)))?;
        
        // Return response
        Ok(Response::new(StoreMetadataResponse { success: true }))
    }
    
    async fn search_similar(
        &self,
        request: Request<SearchSimilarRequest>,
    ) -> Result<Response<SearchSimilarResponse>, Status> {
        let req = request.into_inner();
        
        // Call RAM-Lake search_similar
        let results = self.ram_lake.read().search_similar(
            req.embedding,
            req.limit as usize,
        ).map_err(|e| Status::internal(format!("Failed to search similar: {}", e)))?;
        
        // Convert results
        let proto_results = results.into_iter()
            .map(|(id, score)| search_similar_response::Result {
                id: Some(Uuid { value: id.to_string() }),
                score,
            })
            .collect();
        
        // Return response
        Ok(Response::new(SearchSimilarResponse { 
            results: proto_results,
        }))
    }
    
    async fn get_code(
        &self,
        request: Request<GetCodeRequest>,
    ) -> Result<Response<GetCodeResponse>, Status> {
        let req = request.into_inner();
        
        // Parse UUID
        let code_id = req.id.ok_or_else(|| Status::invalid_argument("Missing code_id"))?;
        let code_uuid = Uuid::parse_str(&code_id.value)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;
        
        // Call RAM-Lake get_code
        let (path, content, language) = self.ram_lake.read().get_code(code_uuid)
            .map_err(|e| Status::internal(format!("Failed to get code: {}", e)))?;
        
        // Return response
        Ok(Response::new(GetCodeResponse {
            path,
            content,
            language,
        }))
    }
    
    async fn get_event(
        &self,
        request: Request<GetEventRequest>,
    ) -> Result<Response<GetEventResponse>, Status> {
        let req = request.into_inner();
        
        // Parse UUID
        let event_id = req.id.ok_or_else(|| Status::invalid_argument("Missing event_id"))?;
        let event_uuid = Uuid::parse_str(&event_id.value)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;
        
        // Call RAM-Lake get_event
        let (event_type, content, timestamp) = self.ram_lake.read().get_event(event_uuid)
            .map_err(|e| Status::internal(format!("Failed to get event: {}", e)))?;
        
        // Convert timestamp
        let proto_timestamp = prost_types::Timestamp {
            seconds: timestamp.timestamp(),
            nanos: timestamp.timestamp_subsec_nanos() as i32,
        };
        
        // Return response
        Ok(Response::new(GetEventResponse {
            event_type,
            content,
            timestamp: Some(proto_timestamp),
        }))
    }
    
    async fn get_related(
        &self,
        request: Request<GetRelatedRequest>,
    ) -> Result<Response<GetRelatedResponse>, Status> {
        let req = request.into_inner();
        
        // Parse UUID
        let id = req.id.ok_or_else(|| Status::invalid_argument("Missing id"))?;
        let uuid = Uuid::parse_str(&id.value)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;
        
        // Call RAM-Lake get_related
        let relations = self.ram_lake.read().get_related(uuid, req.relation.as_deref())
            .map_err(|e| Status::internal(format!("Failed to get related: {}", e)))?;
        
        // Convert relations
        let proto_relations = relations.into_iter()
            .map(|(source_id, relation, target_id)| get_related_response::Relation {
                source_id: Some(Uuid { value: source_id.to_string() }),
                relation,
                target_id: Some(Uuid { value: target_id.to_string() }),
            })
            .collect();
        
        // Return response
        Ok(Response::new(GetRelatedResponse {
            relations: proto_relations,
        }))
    }
    
    async fn get_ram_lake_metrics(
        &self,
        _request: Request<()>,
    ) -> Result<Response<RamLakeMetricsResponse>, Status> {
        // Get metrics from RAM-Lake
        let metrics = self.ram_lake.read().get_metrics();
        
        // Convert metrics
        let response = RamLakeMetricsResponse {
            total_size: metrics.total_size,
            used_size: metrics.used_size,
            vector_store_size: metrics.vector_store_size,
            code_store_size: metrics.code_store_size,
            history_store_size: metrics.history_store_size,
            metadata_store_size: metrics.metadata_store_size,
            indexed_files: metrics.indexed_files as u32,
            vector_entries: metrics.vector_entries as u32,
            history_events: metrics.history_events as u32,
        };
        
        // Return response
        Ok(Response::new(response))
    }
    
    // MLX model operations
    
    async fn load_model(
        &self,
        request: Request<LoadModelRequest>,
    ) -> Result<Response<LoadModelResponse>, Status> {
        let req = request.into_inner();
        
        // Call model manager load_model
        let success = self.model_manager.write().load_model(&req.model_name)
            .map_err(|e| Status::internal(format!("Failed to load model: {}", e)))?;
        
        // Return response
        if success {
            Ok(Response::new(LoadModelResponse {
                success: true,
                error_message: None,
                alternative_model: None,
            }))
        } else {
            // Try to find alternative model
            let alternative = self.model_manager.read().find_alternative_model(&req.model_name)
                .map_err(|e| Status::internal(format!("Failed to find alternative model: {}", e)))?;
            
            Ok(Response::new(LoadModelResponse {
                success: false,
                error_message: Some("Insufficient memory to load model".to_string()),
                alternative_model: alternative,
            }))
        }
    }
    
    async fn unload_model(
        &self,
        request: Request<UnloadModelRequest>,
    ) -> Result<Response<UnloadModelResponse>, Status> {
        let req = request.into_inner();
        
        // Call model manager unload_model
        let success = self.model_manager.write().unload_model(&req.model_name)
            .map_err(|e| Status::internal(format!("Failed to unload model: {}", e)))?;
        
        // Return response
        if success {
            Ok(Response::new(UnloadModelResponse {
                success: true,
                error_message: None,
            }))
        } else {
            Ok(Response::new(UnloadModelResponse {
                success: false,
                error_message: Some("Model not loaded or cannot be unloaded".to_string()),
            }))
        }
    }
    
    async fn run_inference(
        &self,
        request: Request<RunInferenceRequest>,
    ) -> Result<Response<RunInferenceResponse>, Status> {
        let req = request.into_inner();
        
        // Call model manager run_inference
        let result = self.model_manager.write().run_inference(
            &req.model_name,
            &req.input_text,
            req.max_tokens.unwrap_or(512) as usize,
            req.temperature.unwrap_or(0.7),
            req.top_p.unwrap_or(0.9),
            req.frequency_penalty.unwrap_or(0.0),
            req.presence_penalty.unwrap_or(0.0),
        ).map_err(|e| Status::internal(format!("Failed to run inference: {}", e)))?;
        
        // Check for error
        if let Some(error) = result.get("error") {
            return Ok(Response::new(RunInferenceResponse {
                model_name: req.model_name,
                generated_text: String::new(),
                tokens_generated: 0,
                error_message: Some(error.to_string()),
            }));
        }
        
        // Extract result
        let generated_text = result.get("generated_text")
            .ok_or_else(|| Status::internal("No generated text in result"))?
            .to_string();
        
        let tokens_generated = result.get("tokens_generated")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as i32;
        
        // Return response
        Ok(Response::new(RunInferenceResponse {
            model_name: result.get("model").unwrap_or(&req.model_name).to_string(),
            generated_text,
            tokens_generated,
            error_message: None,
        }))
    }
    
    async fn generate_embedding(
        &self,
        request: Request<GenerateEmbeddingRequest>,
    ) -> Result<Response<GenerateEmbeddingResponse>, Status> {
        let req = request.into_inner();
        
        // Use default model if not specified
        let model_name = req.model_name.unwrap_or_else(|| "embedder".to_string());
        
        // Call model manager generate_embedding
        let result = self.model_manager.write().generate_embedding(
            &model_name,
            &req.text,
        ).map_err(|e| Status::internal(format!("Failed to generate embedding: {}", e)))?;
        
        // Check for error
        if let Some(error) = result.get("error") {
            return Ok(Response::new(GenerateEmbeddingResponse {
                embedding: Vec::new(),
                error_message: Some(error.to_string()),
            }));
        }
        
        // Extract embeddings
        let embeddings = result.get("embeddings")
            .ok_or_else(|| Status::internal("No embeddings in result"))?
            .as_array()
            .ok_or_else(|| Status::internal("Embeddings is not an array"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        
        // Return response
        Ok(Response::new(GenerateEmbeddingResponse {
            embedding: embeddings,
            error_message: None,
        }))
    }
    
    async fn get_model_status(
        &self,
        request: Request<GetModelStatusRequest>,
    ) -> Result<Response<GetModelStatusResponse>, Status> {
        let req = request.into_inner();
        
        // Call model manager get_model_status
        let status = self.model_manager.read().get_model_status(&req.model_name)
            .map_err(|e| Status::internal(format!("Failed to get model status: {}", e)))?;
        
        // Check if status is empty
        if status.is_empty() {
            return Err(Status::not_found(format!("Model {} not found", req.model_name)));
        }
        
        // Extract status fields
        let model_status = GetModelStatusResponse {
            name: status.get("name").map(|v| v.to_string()).unwrap_or_default(),
            status: status.get("status").map(|v| v.to_string()).unwrap_or_default(),
            r#type: status.get("type").map(|v| v.to_string()).unwrap_or_default(),
            task: status.get("task").map(|v| v.to_string()).unwrap_or_default(),
            memory_gb: status.get("memory_gb").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            last_used: status.get("last_used").and_then(|v| v.as_f64()),
            priority: status.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            error_message: None,
        };
        
        // Return response
        Ok(Response::new(model_status))
    }
    
    async fn get_memory_usage(
        &self,
        _request: Request<()>,
    ) -> Result<Response<MemoryUsageResponse>, Status> {
        // Call model manager get_memory_usage
        let usage = self.model_manager.read().get_memory_usage()
            .map_err(|e| Status::internal(format!("Failed to get memory usage: {}", e)))?;
        
        // Extract usage fields
        let mut model_usage = std::collections::HashMap::new();
        if let Some(model_usage_json) = usage.get("model_usage_gb") {
            if let Some(obj) = model_usage_json.as_object() {
                for (key, value) in obj {
                    if let Some(gb) = value.as_f64() {
                        model_usage.insert(key.clone(), gb as f32);
                    }
                }
            }
        }
        
        let response = MemoryUsageResponse {
            total_limit_gb: usage.get("total_limit_gb").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            current_usage_gb: usage.get("current_usage_gb").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            available_gb: usage.get("available_gb").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            model_usage_gb: model_usage,
        };
        
        // Return response
        Ok(Response::new(response))
    }
    
    async fn get_loaded_models(
        &self,
        _request: Request<()>,
    ) -> Result<Response<LoadedModelsResponse>, Status> {
        // Call model manager get_loaded_models
        let models = self.model_manager.read().get_loaded_models()
            .map_err(|e| Status::internal(format!("Failed to get loaded models: {}", e)))?;
        
        // Return response
        Ok(Response::new(LoadedModelsResponse {
            model_names: models,
        }))
    }
}