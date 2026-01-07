from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/node_field/document_meta/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    l_0_document_version_ = resolve('document_version_')
    l_0_document_date_ = resolve('document_date_')
    l_0_document_config = missing
    try:
        t_1 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    l_0_document_config = environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'config')
    context.vars['document_config'] = l_0_document_config
    context.exported_vars.add('document_config')
    if (context.call(environment.getattr((undefined(name='document_config') if l_0_document_config is missing else l_0_document_config), 'has_meta')) or context.call(environment.getattr((undefined(name='document_config') if l_0_document_config is missing else l_0_document_config), 'has_custom_metadata'))):
        pass
        yield '<sdoc-meta>'
        if environment.getattr(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'config'), 'uid'):
            pass
            yield '<sdoc-meta-label data-testid="document-config-uid-label">UID:</sdoc-meta-label>\n    <sdoc-meta-field data-testid="document-config-uid-field">'
            l_1_field_content = environment.getattr(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'config'), 'uid')
            pass
            template = environment.get_template('components/field/index.jinja', 'components/node_field/document_meta/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_content': l_1_field_content, 'document_config': l_0_document_config, 'document_date_': l_0_document_date_, 'document_version_': l_0_document_version_}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            l_1_field_content = missing
            yield '</sdoc-meta-field>'
        l_0_document_version_ = context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_document_version'))
        context.vars['document_version_'] = l_0_document_version_
        context.exported_vars.add('document_version_')
        if (not t_1((undefined(name='document_version_') if l_0_document_version_ is missing else l_0_document_version_))):
            pass
            yield '<sdoc-meta-label data-testid="document-config-version-label">VERSION:</sdoc-meta-label>\n    <sdoc-meta-field data-testid="document-config-version-field">'
            l_1_field_content = (undefined(name='document_version_') if l_0_document_version_ is missing else l_0_document_version_)
            pass
            template = environment.get_template('components/field/index.jinja', 'components/node_field/document_meta/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_content': l_1_field_content, 'document_config': l_0_document_config, 'document_date_': l_0_document_date_, 'document_version_': l_0_document_version_}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            l_1_field_content = missing
            yield '</sdoc-meta-field>\n    \n    '
        l_0_document_date_ = context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_document_date'))
        context.vars['document_date_'] = l_0_document_date_
        context.exported_vars.add('document_date_')
        if (not t_1((undefined(name='document_date_') if l_0_document_date_ is missing else l_0_document_date_))):
            pass
            yield '<sdoc-meta-label data-testid="document-config-date-label">DATE:</sdoc-meta-label>\n    <sdoc-meta-field data-testid="document-config-date-field">'
            l_1_field_content = (undefined(name='document_date_') if l_0_document_date_ is missing else l_0_document_date_)
            pass
            template = environment.get_template('components/field/index.jinja', 'components/node_field/document_meta/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_content': l_1_field_content, 'document_config': l_0_document_config, 'document_date_': l_0_document_date_, 'document_version_': l_0_document_version_}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            l_1_field_content = missing
            yield '</sdoc-meta-field>'
        if environment.getattr(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'config'), 'classification'):
            pass
            yield '<sdoc-meta-label data-testid="document-config-classification-label">CLASSIFICATION:</sdoc-meta-label>\n    <sdoc-meta-field data-testid="document-config-classification-field">'
            l_1_field_content = environment.getattr(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'config'), 'classification')
            pass
            template = environment.get_template('components/field/index.jinja', 'components/node_field/document_meta/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'field_content': l_1_field_content, 'document_config': l_0_document_config, 'document_date_': l_0_document_date_, 'document_version_': l_0_document_version_}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            l_1_field_content = missing
            yield '</sdoc-meta-field>'
        if context.call(environment.getattr((undefined(name='document_config') if l_0_document_config is missing else l_0_document_config), 'has_custom_metadata')):
            pass
            for (l_1_key, l_1_value) in context.call(environment.getattr((undefined(name='document_config') if l_0_document_config is missing else l_0_document_config), 'get_custom_metadata')):
                _loop_vars = {}
                pass
                yield '\n    <sdoc-meta-label data-testid="document-config-metadata-label">'
                yield escape(l_1_key)
                yield ':</sdoc-meta-label>\n    <sdoc-meta-field data-testid="document-config-metadata-field">\n      <sdoc-autogen>'
                yield escape(l_1_value)
                yield '</sdoc-autogen>\n    </sdoc-meta-field>\n    '
            l_1_key = l_1_value = missing
            yield '\n    '
        yield '\n\n  </sdoc-meta>'

blocks = {}
debug_info = '1=21&3=24&5=27&9=32&14=40&15=43&19=48&30=56&31=59&35=64&40=72&44=77&49=85&50=87&51=91&53=93'