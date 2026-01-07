from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'actions/document/create_section/stream_cancel_new_section.jinja.html'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_section_mid = resolve('section_mid')
    pass
    yield '<turbo-stream action="replace" target="article-'
    yield escape((undefined(name='section_mid') if l_0_section_mid is missing else l_0_section_mid))
    yield '">\n  <template></template>\n</turbo-stream>'

blocks = {}
debug_info = '1=13'