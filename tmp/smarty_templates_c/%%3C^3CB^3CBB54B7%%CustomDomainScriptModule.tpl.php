<?php /* Smarty version 2.6.7, created on 2008-12-06 17:18:19
         compiled from /var/www/wikidot/templates/modules/login/CustomDomainScriptModule.tpl */ ?>
<?php if ($this->_tpl_vars['useCustomDomainScript']): ?>
	<script type="text/javascript" src="http<?php if ($this->_tpl_vars['useCustomDomainScriptSecure']): ?>s<?php endif; ?>://<?php echo $this->_tpl_vars['URL_HOST']; ?>
/default__flow/login__CustomDomainScript?site_id=<?php echo $this->_tpl_vars['site']->getSiteId(); ?>
"></script>
<?php endif; ?>