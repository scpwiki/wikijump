# resource "aws_ecs_service" "wikijump" {
#     name    = "wikijump-${var.environment}"
#     cluster = aws_ecs_cluster.wikijump_ecs_cluster.id
#     task_definition = aws_ecs_task_definition.wikijump_task.arn
#     desired_count   = 1  # This will be a var as we grow

#     load_balancer {
#         target_group_arn    = aws_lb_target_group.elb_target_group.arn
#         container_name      = "wikijump"
#         container_port      = 80
#     }
#     depends_on = [aws_ecs_cluster.wikijump_ecs_cluster]
# }

# resource "aws_ecs_task_definition" "wikijump_task" {
#     family  = "wikijump-${var.environment}-family"
#     container_definitions   = file("task-definitions/wikijump.json")
#     requires_compatibilities    = ["FARGATE"]
#     cpu                         = 256  # 1024 = 1 vCPU
#     memory                      = 512  # MiB
# }

# resource "aws_ecs_cluster" "wikijump_ecs_cluster" {
#   name                  = "wikijump-${var.environment}"
#   # Using Spot as a cost-saving measure for now. This will end up being dependent on environment.
#   capacity_providers    = ["FARGATE_SPOT"]
# }
