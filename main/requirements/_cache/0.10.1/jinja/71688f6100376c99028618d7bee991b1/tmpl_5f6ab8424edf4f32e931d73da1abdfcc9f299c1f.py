from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/node_content/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_sdoc_entity = resolve('sdoc_entity')
    l_0_requirement_style = resolve('requirement_style')
    l_0_user_requirement_style = l_0_title_number = l_0_truncated_statement = missing
    try:
        t_1 = environment.filters['d']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No filter named 'd' found.")
    pass
    yield '\n\n\n\n\n'
    l_0_user_requirement_style = context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_requirement_style_mode'))
    context.vars['user_requirement_style'] = l_0_user_requirement_style
    context.exported_vars.add('user_requirement_style')
    yield '\n\n<sdoc-node-content\n  node-view="'
    yield escape(t_1((undefined(name='requirement_style') if l_0_requirement_style is missing else l_0_requirement_style), context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_requirement_style_mode'))))
    yield '"\n  data-level="'
    yield escape(environment.getattr(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'context'), 'title_number_string'))
    yield '"'
    if environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_status'):
        pass
        yield "\n    data-status='"
        yield escape(context.call(environment.getattr(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_status'), 'lower')))
        yield "'"
    yield '\n  show-node-type-name="'
    yield escape(context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_node_type_string')))
    yield '"\n  data-testid="requirement-style-'
    yield escape(t_1((undefined(name='requirement_style') if l_0_requirement_style is missing else l_0_requirement_style), context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_requirement_style_mode'))))
    yield '"\n>\n  '
    l_0_title_number = True
    context.vars['title_number'] = l_0_title_number
    context.exported_vars.add('title_number')
    yield '\n  '
    l_0_truncated_statement = False
    context.vars['truncated_statement'] = l_0_truncated_statement
    context.exported_vars.add('truncated_statement')
    yield '\n  '
    template = environment.get_template('components/node_field/title/index.jinja', 'components/node_content/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n  \n  \n\n  '
    if ((undefined(name='user_requirement_style') if l_0_user_requirement_style is missing else l_0_user_requirement_style) == 'narrative'):
        pass
        yield '\n    <sdoc-scope class="node_fields_group-secondary">\n      '
        template = environment.get_template('components/node_field/meta/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n      '
        template = environment.get_template('components/node_field/links/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n      '
        template = environment.get_template('components/node_field/files/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    </sdoc-scope>\n    '
        if context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'has_multiline_fields')):
            pass
            yield '\n    <sdoc-scope class="node_fields_group-primary">\n      '
            template = environment.get_template('components/node_field/statement/index.jinja', 'components/node_content/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '\n      '
            template = environment.get_template('components/node_field/rationale/index.jinja', 'components/node_content/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '\n      '
            template = environment.get_template('components/node_field/comments/index.jinja', 'components/node_content/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '\n      '
            template = environment.get_template('components/node_field/multiline/index.jinja', 'components/node_content/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '\n    </sdoc-scope>\n    '
        yield '\n  '
    elif ((undefined(name='user_requirement_style') if l_0_user_requirement_style is missing else l_0_user_requirement_style) == 'plain'):
        pass
        yield '\n    '
        template = environment.get_template('components/node_field/statement/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/rationale/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/comments/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/multiline/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n  '
    else:
        pass
        yield '\n    '
        template = environment.get_template('components/node_field/meta/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/statement/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/rationale/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/comments/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/multiline/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/links/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    '
        template = environment.get_template('components/node_field/files/index.jinja', 'components/node_content/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'title_number': l_0_title_number, 'truncated_statement': l_0_truncated_statement, 'user_requirement_style': l_0_user_requirement_style}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n  '
    yield '\n\n</sdoc-node-content>'

blocks = {}
debug_info = '18=21&21=25&22=27&23=29&24=32&26=35&27=37&29=39&30=43&31=47&39=54&41=57&42=64&43=71&45=78&47=81&48=88&49=95&50=102&53=110&54=113&55=120&56=127&57=134&59=144&60=151&61=158&62=165&63=172&64=179&65=186'