from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/project_index/project_map_node.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_node = resolve('node')
    l_0_mid = l_0_uid = missing
    try:
        t_1 = environment.tests['defined']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No test named 'defined' found.")
    pass
    def macro():
        t_2 = []
        pass
        return concat(t_2)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, t_1((undefined(name='node') if l_0_node is missing else l_0_node)), None, caller=caller)
    l_0_mid = ''
    context.vars['mid'] = l_0_mid
    context.exported_vars.add('mid')
    if ((t_1(environment.getattr((undefined(name='node') if l_0_node is missing else l_0_node), 'reserved_mid')) and t_1(environment.getattr((undefined(name='node') if l_0_node is missing else l_0_node), 'mid_permanent'))) and environment.getattr((undefined(name='node') if l_0_node is missing else l_0_node), 'mid_permanent')):
        pass
        l_0_mid = environment.getattr((undefined(name='node') if l_0_node is missing else l_0_node), 'reserved_mid')
        context.vars['mid'] = l_0_mid
        context.exported_vars.add('mid')
    l_0_uid = ''
    context.vars['uid'] = l_0_uid
    context.exported_vars.add('uid')
    if t_1(environment.getattr((undefined(name='node') if l_0_node is missing else l_0_node), 'reserved_uid')):
        pass
        l_0_uid = environment.getattr((undefined(name='node') if l_0_node is missing else l_0_node), 'reserved_uid')
        context.vars['uid'] = l_0_uid
        context.exported_vars.add('uid')
    if (undefined(name='uid') if l_0_uid is missing else l_0_uid):
        pass
        yield '  {'
        if (undefined(name='mid') if l_0_mid is missing else l_0_mid):
            pass
            yield '"MID":"'
            yield escape((undefined(name='mid') if l_0_mid is missing else l_0_mid))
            yield '",'
        yield '"UID":"'
        yield escape((undefined(name='uid') if l_0_uid is missing else l_0_uid))
        yield '"},\n'

blocks = {}
debug_info = '1=19&2=25&3=28&4=30&7=33&8=36&9=38&12=41'