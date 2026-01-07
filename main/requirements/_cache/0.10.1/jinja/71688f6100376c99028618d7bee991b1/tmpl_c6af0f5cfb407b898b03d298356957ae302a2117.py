from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/git/fields/section_fields.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_section = resolve('section')
    l_0_section_change = resolve('section_change')
    l_0_uid_modified = resolve('uid_modified')
    l_0_side = resolve('side')
    l_0_title_modified = missing
    try:
        t_1 = environment.filters['length']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No filter named 'length' found.")
    try:
        t_2 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_2(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    yield '<div class="diff_node_fields">\n  \n  '
    if environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'mid_permanent'):
        pass
        yield '\n    <div\n      class="diff_node_field"\n    >'
        l_1_badge_text = 'MID'
        pass
        template = environment.get_template('components/badge/index.jinja', 'screens/git/fields/section_fields.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'badge_text': l_1_badge_text, 'title_modified': l_0_title_modified, 'uid_modified': l_0_uid_modified}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        l_1_badge_text = missing
        yield '<span class="sdoc_pre_content">'
        yield escape(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'reserved_mid'))
        yield '</span>\n    </div>\n  '
    yield '\n\n  '
    if ((not t_2(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'reserved_uid'))) and (t_1(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'reserved_uid')) > 0)):
        pass
        l_0_uid_modified = ((not t_2((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change))) and environment.getattr((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change), 'uid_modified'))
        context.vars['uid_modified'] = l_0_uid_modified
        context.exported_vars.add('uid_modified')
        yield '\n    <div\n      class="diff_node_field"\n      '
        if (undefined(name='uid_modified') if l_0_uid_modified is missing else l_0_uid_modified):
            pass
            yield '\n        modified="'
            yield escape((undefined(name='side') if l_0_side is missing else l_0_side))
            yield '"\n      '
        yield '\n    >'
        l_1_badge_text = 'UID'
        pass
        template = environment.get_template('components/badge/index.jinja', 'screens/git/fields/section_fields.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'badge_text': l_1_badge_text, 'title_modified': l_0_title_modified, 'uid_modified': l_0_uid_modified}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        l_1_badge_text = missing
        yield '<div class="sdoc_pre_content">'
        yield escape(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'reserved_uid'))
        yield '</div>\n    </div>'
    yield '\n\n  \n  '
    l_0_title_modified = ((not t_2((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change))) and environment.getattr((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change), 'title_modified'))
    context.vars['title_modified'] = l_0_title_modified
    context.exported_vars.add('title_modified')
    yield '\n  <div\n    class="diff_node_field"\n    '
    if (undefined(name='title_modified') if l_0_title_modified is missing else l_0_title_modified):
        pass
        yield '\n      modified="'
        yield escape((undefined(name='side') if l_0_side is missing else l_0_side))
        yield '"\n    '
    yield '\n  >'
    l_1_badge_text = 'TITLE'
    pass
    template = environment.get_template('components/badge/index.jinja', 'screens/git/fields/section_fields.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'badge_text': l_1_badge_text, 'title_modified': l_0_title_modified, 'uid_modified': l_0_uid_modified}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    l_1_badge_text = missing
    if (undefined(name='title_modified') if l_0_title_modified is missing else l_0_title_modified):
        pass
        if context.call(environment.getattr((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change), 'is_paired_change')):
            pass
            yield '<div class="sdoc_pre_content">'
            yield escape(context.call(environment.getattr((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change), 'get_colored_title_diff'), (undefined(name='side') if l_0_side is missing else l_0_side)))
            yield '</div>'
        else:
            pass
            yield '<div class="sdoc_pre_content">'
            yield escape(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'reserved_title'))
            yield '</div>'
    else:
        pass
        yield '\n    <div class="sdoc_pre_content">'
        yield escape(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'reserved_title'))
        yield '</div>\n    '
    yield '\n  </div>\n\n</div>'

blocks = {}
debug_info = '3=29&8=34&10=42&15=45&16=47&19=51&20=54&24=59&26=67&31=70&34=74&35=77&39=82&41=89&42=91&43=94&45=99&48=104'