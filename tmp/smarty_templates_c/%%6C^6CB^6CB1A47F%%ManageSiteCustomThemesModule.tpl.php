<?php /* Smarty version 2.6.7, created on 2009-01-04 19:48:13
         compiled from /var/www/wikidot/templates/modules/managesite/ManageSiteCustomThemesModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('modifier', 'escape', '/var/www/wikidot/templates/modules/managesite/ManageSiteCustomThemesModule.tpl', 17, false),)), $this); ?>
<h1>Custom themes</h1>

<p>
	Using custom themes you can easily convert your site into a brand new quality. 
	The idea is simple - you can create your own CSS rules. <br/>
	<!-- TODO: De-Wikidot.com-ize - change -->
	Look at the <a href="http://www.wikidot.com/doc:layout-reference" target="_blank">CSS layout reference too</a> and
	<!-- TODO: De-Wikidot.com-ize - change -->
	<a href="http://community.wikidot.com/howto:design-your-own-css-theme"  target="_blank">Design 
	your own CSS theme</a> howto.
</p>

<?php if ($this->_tpl_vars['themes']): ?>
	<ul>
		<?php if (count($_from = (array)$this->_tpl_vars['themes'])):
    foreach ($_from as $this->_tpl_vars['theme']):
?>
			<li>
				<?php echo ((is_array($_tmp=$this->_tpl_vars['theme']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>

				(<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteCustomThemesModule.listeners.editTheme(event, <?php echo $this->_tpl_vars['theme']->getThemeId(); ?>
)">edit</a>
				| <a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteCustomThemesModule.listeners.deleteTheme(event, <?php echo $this->_tpl_vars['theme']->getThemeId(); ?>
)">delete</a>)
			</li>
		<?php endforeach; endif; unset($_from); ?>
	</ul>
<?php else: ?>
	<p>
		There are no custom themes for this site.
	</p>
<?php endif; ?>

<p>
	<a class="button" href="javascript:;" onclick="WIKIDOT.modules.ManageSiteCustomThemesModule.listeners.editTheme(event)">create a new theme</a>
</p>

<div id="edit-theme-box"></div>
