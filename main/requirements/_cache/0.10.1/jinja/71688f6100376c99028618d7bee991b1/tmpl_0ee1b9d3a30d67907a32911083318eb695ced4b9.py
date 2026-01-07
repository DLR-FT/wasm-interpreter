from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/issue/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_issue_field_name = resolve('issue_field_name')
    pass
    yield '\n\n\n<div id="field_issue_ID" class="field_issue">\n  <div class="field_issue-ribbon">\n    <b>Warning:</b> \''
    yield escape((undefined(name='issue_field_name') if l_0_issue_field_name is missing else l_0_issue_field_name))
    yield "' field has issue: description in details\n  </div>\n</div>"

blocks = {}
debug_info = '10=13'