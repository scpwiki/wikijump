#####
# Execution IAM Role
#####
resource "aws_iam_role" "execution" {
  name               = "wj-${var.environment}-execution-role"
  assume_role_policy = data.aws_iam_policy_document.assume_role_policy.json

  tags = var.tags
}

resource "aws_iam_role_policy_attachment" "ecs_task_execution_role_policy_attach" {
  role       = aws_iam_role.execution[0].name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role_policy" "read_repository_credentials" {
  name   = "wj-${var.environment}-read-repository-credentials"
  role   = aws_iam_role.execution[0].id
  policy = data.aws_iam_policy_document.read_repository_credentials[0].json
}

#####
# IAM - Task role, basic. Append policies to this role for S3, DynamoDB etc.
#####
resource "aws_iam_role" "task" {
  name               = "wj-${var.environment}-task-role"
  assume_role_policy = data.aws_iam_policy_document.assume_role_policy.json

  tags = var.tags
}

resource "aws_iam_role_policy" "log_agent" {
  name   = "wj-${var.environment}-log-permissions"
  role   = aws_iam_role.task[0].id
  policy = data.aws_iam_policy_document.task_permissions.json
}
