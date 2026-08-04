MODULE ExtensionPrivate;
  IMPORT ExtensionPrivateSource;
  TYPE Bad = RECORD (ExtensionPrivateSource.Hidden) END;
  VAR child: ExtensionPrivateSource.Child; n: INTEGER;
BEGIN
  n := child.private
END ExtensionPrivate.
