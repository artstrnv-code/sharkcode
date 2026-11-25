"""Validator of the creative library project (called by the trusted runner)."""


def validate(context):
    from project_validator import inspect_project
    return inspect_project(context.get('project_files', {}))
