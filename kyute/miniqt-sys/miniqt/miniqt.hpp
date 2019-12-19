#ifndef MINIQT_H
#define MINIQT_H
#include <cstdint>

#include <QtCore>
#include <QtGui>
#include <QtWidgets>

// provide our own destructors since bindgen can't generate bindings
// to inline or implicitly-declared destructors.

//====== QtCore ======
// QAbstractAnimation
// QAbstractEventDispatcher
// QAbstractItemModel
// QAbstractListModel
// QAbstractNativeEventFilter
// QAbstractProxyModel
// QAbstractState
// QAbstractTableModel
// QAbstractTransition
// QAnimationGroup
// QAssociativeIterable
// QAtomicInt
// QAtomicInteger
// QAtomicPointer
// QBEInteger
// QBasicTimer
// QBitArray
// QBuffer
// QByteArray
QByteArray *QByteArray_new();
void QByteArray_destructor(QByteArray *byteArray);
void QByteArray_delete(QByteArray *byteArray);

// QByteArrayList
// QByteArrayMatcher
// QCache
// QChar
// QChildEvent
// QCollator
// QCollatorSortKey
// QCommandLineOption
// QCommandLineParser
// QContiguousCache
// QCoreApplication
void QCoreApplication_processEvents(QEventLoop::ProcessEventsFlags flags);
void QCoreApplication_installEventFilter(QObject *filterObj);
void QCoreApplication_removeEventFilter(QObject *filterObj);
// QCryptographicHash
// QDataStream
// QDate
// QDateTime
// QDeadlineTimer
// QDebug
// QDebugStateSaver
// QDir
// QDirIterator
// QDynamicPropertyChangeEvent
// QEasingCurve
// QElapsedTimer
// QEnableSharedFromThis
// QEvent
// QEventLoop
// QEventLoopLocker
// QEventTransition
// QException
// QExplicitlySharedDataPointer
// QFile
// QFileDevice
// QFileInfo
// QFileSelector
// QFileSystemWatcher
// QFinalState
// QFlag
// QFlags
// QFuture
// QFutureIterator
// QFutureSynchronizer
// QFutureWatcher
// QGenericArgument
// QGenericReturnArgument
// QGlobalStatic
// QHash
// QHashIterator
// QHistoryState
// QIODevice
// QIdentityProxyModel
// QItemSelection
// QItemSelectionModel
// QItemSelectionRange
// QJsonArray
// QJsonDocument
// QJsonObject
// QJsonParseError
// QJsonValue
// QKeyValueIterator
// QLEInteger
// QLatin1Char
// QLatin1String
// QLibrary
// QLibraryInfo
// QLine
// QLineF
// QLinkedList
// QLinkedListIterator
// QList
// QListIterator
// QLocale
// QLockFile
// QLoggingCategory
// QMap
// QMapIterator
// QMargins
// QMarginsF
// QMessageAuthenticationCode
// QMessageLogContext
// QMessageLogger
// QMetaClassInfo
// QMetaEnum
// QMetaMethod
// QMetaObject
// QMetaProperty
// QMetaType
// QMimeData
// QMimeDatabase
// QMimeType
// QModelIndex
// QMultiHash
// QMultiMap
// QMutableHashIterator
// QMutableLinkedListIterator
// QMutableListIterator
// QMutableMapIterator
// QMutableSetIterator
// QMutableVectorIterator
// QMutex
// QMutexLocker
// QObject
void QObject_destructor(QObject *object);
void QObject_delete(QObject *object);
void QObject_connect_abi(const QObject *sender, const char *signal,
                         const QObject *receiver, const char *method,
                         Qt::ConnectionType type); // ABI-safe replacement
void QObject_installEventFilter(QObject *self, QObject *filterObj);
void QObject_removeEventFilter(QObject *self, QObject *filterObj);
bool QObject_setProperty(QObject *self, const char *name, const QVariant &value);
void QObject_property(const QObject *self, const char *name, QVariant& outVariant);
bool QObject_property_uint64(const QObject *self, const char *name, uint64_t& outValue);
bool QObject_setProperty_uint64(QObject *self, const char *name, uint64_t value);
QWidget* QObject_downcast_QWidget(QObject* self);

// QObjectCleanupHandler
// QOperatingSystemVersion
// QPair
// QParallelAnimationGroup
// QPauseAnimation
// QPersistentModelIndex
// QPluginLoader
// QPoint
// QPointF
// QPointer
// QProcess
// QProcessEnvironment
// QPropertyAnimation
// QQueue
// QRandomGenerator
// QRandomGenerator64
// QReadLocker
// QReadWriteLock
// QRect
void QRect_getCoords(const QRect* rect, int *x1, int *y1, int *x2, int *y2);
// QRectF
void QRectF_constructor(QRectF* rect, qreal x, qreal y, qreal w, qreal h);
// QRegExp
// QRegularExpression
// QRegularExpressionMatch
// QRegularExpressionMatchIterator
// QResource
// QRunnable
// QSaveFile
// QScopedArrayPointer
// QScopedPointer
// QScopedValueRollback
// QSemaphore
// QSemaphoreReleaser
// QSequentialAnimationGroup
// QSequentialIterable
// QSet
// QSetIterator
// QSettings
// QSharedData
// QSharedDataPointer
// QSharedMemory
// QSharedPointer
// QSignalBlocker
// QSignalTransition
// QSize
// QSizeF
// QSocketNotifier
// QSortFilterProxyModel
// QStack
// QStandardPaths
// QState
// QStateMachine
// QStaticByteArrayMatcher
// QStaticPlugin
// QStorageInfo
// QString
void QString_constructor(QString *ptr);
void QString_destructor(QString *string);
int QString_size(const QString *string);
const uint16_t* QString_utf16(const QString *string);
void QString_fromUtf8(const char *str, int size,
                      QString &out); // ABI-safe replacement

// QStringList
QStringList *QStringList_new();
void QStringList_destructor(QStringList *stringList);
void QStringList_delete(QStringList *stringList);

// QStringListModel
// QStringMatcher
// QStringRef
// QStringView
// QSysInfo
// QSystemSemaphore
// QTemporaryDir
// QTemporaryFile
// QTextBoundaryFinder
// QTextCodec
// QTextDecoder
// QTextEncoder
// QTextStream
// QThread
// QThreadPool
// QThreadStorage
// QTime
// QTimeLine
// QTimeZone
// QTimer
// QTimerEvent
// QTranslator
// QUnhandledException
// QUrl
// QUrlQuery
// QUuid
// QVarLengthArray
// QVariant
void QVariant_constructor_quint64(QVariant *variant, quint64 v);
void QVariant_destructor(QVariant *variant);
// QVariantAnimation
// QVector
// QVectorIterator
// QVersionNumber
// QWaitCondition
// QWeakPointer
// QWinEventNotifier
// QWriteLocker
// QXmlStreamAttribute
// QXmlStreamAttributes
// QXmlStreamEntityDeclaration
// QXmlStreamEntityResolver
// QXmlStreamNamespaceDeclaration
// QXmlStreamNotationDeclaration
// QXmlStreamReader
// QXmlStreamWriter

//====== QtGui ======

// QAccessible
// QAccessibleEditableTextInterface
// QAccessibleEvent
// QAccessibleInterface
// QAccessibleStateChangeEvent
// QAccessibleTableCellInterface
// QAccessibleTableModelChangeEvent
// QAccessibleTextCursorEvent
// QAccessibleTextInsertEvent
// QAccessibleTextInterface
// QAccessibleTextRemoveEvent
// QAccessibleTextSelectionEvent
// QAccessibleTextUpdateEvent
// QAccessibleValueChangeEvent
// QAccessibleValueInterface
// QAccessible
// QAccessibleObject
// QBitmap
// QIcon
// QIconEngine
// QIconEngine
// QIconEngine
// QIconEnginePlugin
// QImage
// QImageIOHandler
// QImageIOPlugin
// QImageReader
// QImageWriter
// QMovie
// QPicture
// QPixmap
void QPixmap_destructor(QPixmap *pixmap);
// QPixmapCache
// QPixmapCache
// QStandardItem
// QStandardItemModel
// QClipboard
// QCursor
// QDrag
// QInputMethodEvent
// QActionEvent
// QCloseEvent
// QContextMenuEvent
// QDragEnterEvent
// QDragLeaveEvent
// QDragMoveEvent
// QDropEvent
// QEnterEvent
// QExposeEvent
// QFileOpenEvent
// QFocusEvent
// QHelpEvent
// QHideEvent
// QHoverEvent
// QIconDragEvent
// QInputEvent
// QInputMethodEvent
// QInputMethodQueryEvent
// QKeyEvent
// QMouseEvent
// QMoveEvent
// QNativeGestureEvent
// QPaintEvent
const QRect& QPaintEvent_rect(const QPaintEvent* paintEvent);
// QPlatformSurfaceEvent
// QPointingDeviceUniqueId
// QResizeEvent
// QScrollEvent
// QScrollPrepareEvent
// QShortcutEvent
// QShowEvent
// QStatusTipEvent
// QTabletEvent
// QTouchEvent
// QWhatsThisClickedEvent
// QWheelEvent
// QWindowStateChangeEvent
// QTouchEvent
// QGenericPlugin
// QGenericPluginFactory
// QGuiApplication
// QInputMethod
// QKeySequence
// QOffscreenSurface
// QOpenGLContext
// QOpenGLContextGroup
// QOpenGLVersionProfile
// QOpenGLWindow
// QPaintDeviceWindow
// QPalette
// QPixelFormat
// QRasterWindow
// QScreen
// QSessionManager
// QStyleHints
// QSurface
// QSurfaceFormat
// QTouchDevice
// QWindow
// QGenericMatrix
// QMatrix4x4
// QQuaternion
// QVector2D
// QVector3D
// QVector4D
// QOpenGLBuffer
// QOpenGLDebugLogger
// QOpenGLDebugMessage
// QOpenGLExtraFunctions
// QOpenGLFramebufferObject
// QOpenGLFramebufferObjectFormat
// QOpenGLFunctions
// QOpenGLFunctions_1_0
// QOpenGLFunctions_3_2_Core
// QOpenGLPaintDevice
// QOpenGLShader
// QOpenGLShaderProgram
// QOpenGLTexture
// QOpenGLTextureBlitter
// QOpenGLTimeMonitor
// QOpenGLTimerQuery
// QAbstractOpenGLFunctions
// QOpenGLVertexArrayObject
// QOpenGLVertexArrayObject
// QBackingStore
// QBrush
void QBrush_constructor(QBrush *brush);
void QBrush_constructor1(QBrush *brush, const QColor& color);
void QBrush_constructor2(QBrush *brush, const QGradient& gradient);
void QBrush_destructor(QBrush *brush);
// QConicalGradient
void QConicalGradient_destructor(QConicalGradient *conicalGradient);
// QGradient
void QGradient_constructor(QGradient* gradient);
void QGradient_destructor(QGradient *gradient);
void QGradient_setSpread(QGradient *gradient, QGradient::Spread spread);
QGradient::Spread QGradient_spread(const QGradient *gradient);
void QGradient_setColorAt(QGradient *gradient, qreal pos, const QColor &color);
QGradient::CoordinateMode QGradient_coordinateMode(const QGradient *gradient);
void QGradient_setCoordinateMode(QGradient *gradient, QGradient::CoordinateMode mode);
QGradient::InterpolationMode QGradient_interpolationMode(const QGradient *gradient);
void QGradient_setInterpolationMode(QGradient *gradient, QGradient::InterpolationMode mode);
// QLinearGradient
void QLinearGradient_constructor(QLinearGradient *linearGradient);
void QLinearGradient_constructor1(QLinearGradient *linearGradient, const QPointF& start, const QPointF& finalStop);
void QLinearGradient_destructor(QLinearGradient *linearGradient);
// QRadialGradient
void QRadialGradient_constructor(QRadialGradient *radialGradient);
void QRadialGradient_destructor(QRadialGradient *radialGradient);
// QColor
void QColor_constructor(QColor *color);
void QColor_destructor(QColor *color);
void QColor_fromRgb(QColor *color, int r, int g, int b, int a);
void QColor_fromRgbF(QColor *color, qreal r, qreal g, qreal b, qreal a);
void QColor_fromRgba64(QColor *color, ushort r, ushort g, ushort b, ushort a);
void QColor_fromHsv(QColor *color, int h, int s, int v, int a);
void QColor_fromHsvF(QColor *color, qreal h, qreal s, qreal v, qreal a);
void QColor_fromCmyk(QColor *color, int c, int m, int y, int k, int a);
void QColor_fromCmykF(QColor *color, qreal c, qreal m, qreal y, qreal k, qreal a);
void QColor_fromHsl(QColor *color, int h, int s, int l, int a);
void QColor_fromHslF(QColor *color, qreal h, qreal s, qreal l, qreal a);
qreal QColor_redF(const QColor* color);
qreal QColor_greenF(const QColor* color);
qreal QColor_blueF(const QColor* color);
qreal QColor_alphaF(const QColor* color);

// QPagedPaintDevice
// QPageLayout
// QPageSize
// QPaintDevice
QWidget* QPaintDevice_downcast_QWidget(QPaintDevice* paintDevice);
// QPaintEngine
// QPaintEngineState
// QTextItem
// QPainter
QPainter *QPainter_new();
void QPainter_constructor(QPainter *painter);
void QPainter_constructor1(QPainter *painter, QPaintDevice* paintDevice);
void QPainter_destructor(QPainter *painter);
void QPainter_delete(QPainter *painter);
void QPainter_setCompositionMode(QPainter* painter, QPainter::CompositionMode mode);
void QPainter_setFont(QPainter* painter, const QFont &f);
void QPainter_setPen(QPainter* painter, const QColor &color);
void QPainter_setPen1(QPainter* painter, const QPen &pen);
void QPainter_setPen2(QPainter* painter, Qt::PenStyle style);
void QPainter_setBrush(QPainter* painter, const QBrush &brush);
void QPainter_setBrush1(QPainter* painter, Qt::BrushStyle style);
void QPainter_setBackgroundMode(QPainter* painter, Qt::BGMode mode);
void QPainter_setBrushOrigin(QPainter* painter, const QPointF &origin);
void QPainter_setBackground(QPainter* painter, const QBrush &bg);
void QPainter_setOpacity(QPainter* painter, qreal opacity);
void QPainter_setClipRect(QPainter* painter, const QRectF &rect, Qt::ClipOperation op);
void QPainter_setClipRegion(QPainter* painter, const QRegion &region, Qt::ClipOperation op);
void QPainter_setClipPath(QPainter* painter, const QPainterPath &path, Qt::ClipOperation op);
void QPainter_setClipping(QPainter* painter, bool enable);
bool QPainter_hasClipping(const QPainter* painter);
void QPainter_save(QPainter* painter);
void QPainter_restore(QPainter* painter);
void QPainter_setTransform(QPainter* painter, const QTransform &transform, bool combine);
void QPainter_resetTransform(QPainter* painter);
void QPainter_setWorldTransform(QPainter* painter, const QTransform &matrix, bool combine);
void QPainter_setWorldMatrixEnabled(QPainter* painter, bool enabled);
void QPainter_scale(QPainter* painter, qreal sx, qreal sy);
void QPainter_shear(QPainter* painter, qreal sh, qreal sv);
void QPainter_rotate(QPainter* painter, qreal a);
void QPainter_translate(QPainter* painter, qreal dx, qreal dy);
void QPainter_setWindow(QPainter* painter, const QRect& rect);
void QPainter_setViewport(QPainter* painter, const QRect& rect);
void QPainter_setViewTransformEnabled(QPainter* painter, bool enable);
void QPainter_strokePath(QPainter* painter, const QPainterPath &path, const QPen &pen);
void QPainter_fillPath(QPainter* painter, const QPainterPath &path, const QBrush &brush);
void QPainter_drawPath(QPainter* painter, const QPainterPath &path);
void QPainter_drawPoint(QPainter* painter, const QPointF& p);
void QPainter_drawLine(QPainter* painter, const QPointF& start, const QPointF& end);
void QPainter_drawRect(QPainter* painter, const QRectF& rect);
void QPainter_drawEllipse(QPainter* painter, const QRectF& rect);
void QPainter_drawEllipse1(QPainter* painter, const QPointF& center, qreal rx, qreal ry);
void QPainter_drawPolyline(QPainter* painter, const QPointF *points, int pointCount);
void QPainter_drawPolygon(QPainter* painter, const QPointF *points, int pointCount, Qt::FillRule fillRule);
void QPainter_drawConvexPolygon(QPainter* painter, const QPointF *points, int pointCount);
void QPainter_drawArc(QPainter* painter, const QRectF& rect, int a, int alen);
void QPainter_drawPie(QPainter* painter, const QRectF& rect, int a, int alen);
void QPainter_drawChord(QPainter* painter, const QRectF& rect, int a, int alen);
void QPainter_drawRoundedRect(QPainter* painter, const QRectF& rect, qreal xRadius, qreal yRadius, Qt::SizeMode mode);
void QPainter_drawTiledPixmap(QPainter* painter, const QRectF& rect, const QPixmap &pm, const QPointF& p);
void QPainter_drawPixmap(QPainter* painter, const QRectF& dst, const QPixmap &pixmap, const QRectF& src);
void QPainter_drawPixmap1(QPainter* painter, const QPointF& dst, const QPixmap &pm);
void QPainter_drawPixmapFragments(QPainter* painter, const QPainter::PixmapFragment *fragments, int fragmentCount, const QPixmap &pixmap, QPainter::PixmapFragmentHints hints);
void QPainter_drawImage(QPainter* painter, const QRectF& dst, const QImage &image, const QRectF& src, Qt::ImageConversionFlags flags);
void QPainter_drawImage1(QPainter* painter, const QPointF& dst, const QImage &image);
void QPainter_setLayoutDirection(QPainter* painter, Qt::LayoutDirection direction);
void QPainter_drawGlyphRun(QPainter* painter, const QPointF& pos, const QGlyphRun &glyphRun);
void QPainter_drawStaticText(QPainter* painter, const QPointF& pos, const QStaticText &staticText);
void QPainter_drawText(QPainter* painter, const QPointF& pos, const QString &s);
void QPainter_drawText1(QPainter* painter, const QRectF& rect, const QString &text, const QTextOption &o);
void QPainter_boundingRect(QPainter* painter, const QRectF& rect, const QString &text, const QTextOption &o, QRectF& out);
void QPainter_setRenderHint(QPainter* painter, QPainter::RenderHint hint, bool on);
void QPainter_setRenderHints(QPainter* painter, QPainter::RenderHints hints, bool on);
void QPainter_beginNativePainting(QPainter* painter);
void QPainter_endNativePainting(QPainter* painter);

// QPainterPath
QPainterPath *QPainterPath_new();
void QPainterPath_constructor(QPainterPath *painterPath);
void QPainterPath_destructor(QPainterPath *painterPath);
void QPainterPath_delete(QPainterPath *painterPath);
void QPainterPath_addEllipse(QPainterPath *painterPath, qreal x, qreal y,
                             qreal width, qreal height);
void QPainterPath_addPath(QPainterPath *painterPath, const QPainterPath &path);
void QPainterPath_addPolygon(QPainterPath *painterPath,
                             const QPolygonF &polygon);
void QPainterPath_addRect(QPainterPath *painterPath, qreal x, qreal y,
                          qreal width, qreal height);
void QPainterPath_addRegion(QPainterPath *painterPath, const QRegion &region);
void QPainterPath_addRoundedRect(QPainterPath *painterPath, qreal x, qreal y,
                                 qreal w, qreal h, qreal xRadius, qreal yRadius,
                                 Qt::SizeMode mode);
void QPainterPath_addText(QPainterPath *painterPath, qreal x, qreal y,
                          const QFont &font, const QString &text);
qreal QPainterPath_angleAtPercent(QPainterPath *painterPath, qreal t);
void QPainterPath_arcMoveTo(QPainterPath *painterPath, qreal x, qreal y,
                            qreal width, qreal height, qreal angle);
void QPainterPath_arcTo(QPainterPath *painterPath, qreal x, qreal y,
                        qreal width, qreal height, qreal startAngle,
                        qreal sweepLength);
void QPainterPath_boundingRect(QPainterPath *painterPath, QRectF &out);
int QPainterPath_capacity(const QPainterPath *painterPath);
void QPainterPath_clear(QPainterPath *painterPath);
void QPainterPath_closeSubpath(QPainterPath *painterPath);
void QPainterPath_connectPath(QPainterPath *painterPath,
                              const QPainterPath &path);
bool QPainterPath_contains(const QPainterPath *painterPath,
                           const QPointF &point);
bool QPainterPath_contains(const QPainterPath *painterPath,
                           const QRectF &rectangle);
bool QPainterPath_contains(const QPainterPath *painterPath,
                           const QPainterPath &p);
void QPainterPath_cubicTo(QPainterPath *painterPath, qreal c1X, qreal c1Y,
                          qreal c2X, qreal c2Y, qreal endPointX,
                          qreal endPointY);
int QPainterPath_elementCount(const QPainterPath *painterPath);
void QPainterPath_intersected(const QPainterPath *painterPath,
                              const QPainterPath &p, QPainterPath &out);
bool QPainterPath_intersects(const QPainterPath *painterPath,
                             const QRectF &rectangle);
bool QPainterPath_intersects(const QPainterPath *painterPath,
                             const QPainterPath &p);
bool QPainterPath_isEmpty(const QPainterPath *painterPath);
qreal QPainterPath_length(const QPainterPath *painterPath);
void QPainterPath_lineTo(QPainterPath *painterPath, qreal x, qreal y);
void QPainterPath_moveTo(QPainterPath *painterPath, qreal x, qreal y);
qreal QPainterPath_percentAtLength(QPainterPath *painterPath, qreal len);
void QPainterPath_quadTo(QPainterPath *painterPath, qreal cx, qreal cy,
                         qreal endPointX, qreal endPointY);
void QPainterPath_reserve(QPainterPath *painterPath, int size);
void QPainterPath_setElementPositionAt(QPainterPath *painterPath, int index,
                                       qreal x, qreal y);
void QPainterPath_setFillRule(QPainterPath *painterPath, Qt::FillRule fillRule);
void QPainterPath_simplified(const QPainterPath *painterPath,
                             QPainterPath &out);
qreal QPainterPath_slopeAtPercent(const QPainterPath *painterPath, qreal t);
void QPainterPath_subtracted(const QPainterPath *painterPath,
                             const QPainterPath &p, QPainterPath &out);
void QPainterPath_swap(QPainterPath *painterPath, QPainterPath &other);
void QPainterPath_toFillPolygon1(QPainterPath *painterPath,
                                 const QMatrix &matrix, QPolygonF &out);
void QPainterPath_toReversed(const QPainterPath *painterPath,
                             QPainterPath &out);
void QPainterPath_translate(QPainterPath *painterPath, qreal dx, qreal dy);
void QPainterPath_translated(const QPainterPath *painterPath, qreal dx,
                             qreal dy, QPainterPath &out);
void QPainterPath_united(const QPainterPath *painterPath, const QPainterPath &p,
                         QPainterPath &out);
// QPainterPathStroker
// QPdfWriter
QPdfWriter *QPdfWriter_new();
void QPdfWriter_destructor(QPdfWriter *pdfWriter);
void QPdfWriter_delete(QPdfWriter *pdfWriter);
// QPen
void QPen_constructor(QPen *pen);
void QPen_destructor(QPen *pen);
void QPen_setWidth(QPen *pen, qreal width);
void QPen_setBrush(QPen *pen, const QBrush& brush);
// QPolygon
void QPolygon_destructor(QPolygon *polygon);
// QPolygonF
void QPolygonF_destructor(QPolygonF *polygon);
// QRegion
void QRegion_destructor(QRegion *region);
// QRgba64
// QTransform
void QTransform_destructor(QTransform *transform);

// QAbstractTextDocumentLayout
// QTextObjectInterface
// QAbstractTextDocumentLayout
// QAbstractTextDocumentLayout
// QFont
void QFont_constructor(QFont* font);
void QFont_destructor(QFont *font);
void QFont_swap(QFont* font, QFont &other);
void QFont_setFamily(QFont* font, const QString &family);
void QFont_setFamilies(QFont* font, const QStringList &families);
void QFont_setStyleName(QFont* font, const QString &styleName);
void QFont_setPointSize(QFont* font, int pointSize);
void QFont_setPointSizeF(QFont* font, qreal pointSize);
void QFont_setPixelSize(QFont* font, int pixelSize);
void QFont_setWeight(QFont* font, int weight);
void QFont_setBold(QFont* font, bool bold);
void QFont_setStyle(QFont* font, QFont::Style style);
void QFont_setItalic(QFont* font, bool italic);
void QFont_setUnderline(QFont* font, bool underline);
void QFont_setOverline(QFont* font, bool overline);
void QFont_setStrikeOut(QFont* font, bool strikeout);
void QFont_setFixedPitch(QFont* font, bool fixedPitch);
void QFont_setKerning(QFont* font, bool kerning);
void QFont_setStyleHint(QFont* font, QFont::StyleHint styleHint, QFont::StyleStrategy styleStrategy);
void QFont_setStyleStrategy(QFont* font, QFont::StyleStrategy s);
void QFont_setStretch(QFont* font, int stretch);
void QFont_setLetterSpacing(QFont* font, QFont::SpacingType type, qreal spacing);
void QFont_setWordSpacing(QFont* font, qreal spacing);
void QFont_setCapitalization(QFont* font, QFont::Capitalization capitalization);
void QFont_setHintingPreference(QFont* font, QFont::HintingPreference hintingPreference);
bool QFont_exactMatch(const QFont* font);
bool QFont_isCopyOf(const QFont* font, const QFont &other);
void QFont_key(const QFont* font, QString& out);
// QFontDatabase
// QFontInfo
void QFontInfo_destructor(QFontInfo *fontInfo);
// QFontMetrics
void QFontMetrics_destructor(QFontMetrics *fontMetrics);
// QFontMetricsF
void QFontMetricsF_destructor(QFontMetricsF *fontMetrics);
// QGlyphRun
void QGlyphRun_destructor(QGlyphRun *glyphRun);
// QRawFont
// QStaticText
void QStaticText_destructor(QStaticText *staticText);
// QSyntaxHighlighter
// QTextCursor
// QTextDocument
void QTextDocument_destructor(QTextDocument *textDocument);
// QTextDocumentFragment
// QTextDocumentWriter
// QTextBlockFormat
// QTextCharFormat
// QTextFormat
// QTextFrameFormat
// QTextImageFormat
// QTextLength
// QTextListFormat
// QTextTableCellFormat
// QTextTableFormat
// QTextInlineObject
// QTextLayout
// QTextLine
// QTextLayout
// QTextList
// QTextBlock
// QTextBlockGroup
// QTextBlockUserData
// QTextFragment
// QTextFrame
// QTextObject
// QTextBlock
// QTextOption
// QTextOption
// QTextTable
// QTextTableCell
// QDesktopServices
// QDoubleValidator
// QIntValidator
// QRegExpValidator
// QValidator
// QVulkanInstance
// QRasterPaintEngine
// QSupportedWritingSystems
// QOpenGLFunctions_1_1
// QOpenGLFunctions_1_2
// QOpenGLFunctions_1_3
// QOpenGLFunctions_1_4
// QOpenGLFunctions_1_5
// QOpenGLFunctions_2_0
// QOpenGLFunctions_2_1
// QOpenGLFunctions_3_0
// QOpenGLFunctions_3_1
// QOpenGLFunctions_3_2_Compatibility
// QOpenGLFunctions_3_3_Compatibility
// QOpenGLFunctions_3_3_Core
// QOpenGLFunctions_4_0_Compatibility
// QOpenGLFunctions_4_0_Core
// QOpenGLFunctions_4_1_Compatibility
// QOpenGLFunctions_4_1_Core
// QOpenGLFunctions_4_2_Compatibility
// QOpenGLFunctions_4_2_Core
// QOpenGLFunctions_4_3_Compatibility
// QOpenGLFunctions_4_3_Core
// QOpenGLFunctions_4_4_Compatibility
// QOpenGLFunctions_4_4_Core
// QOpenGLFunctions_4_5_Compatibility
// QOpenGLFunctions_4_5_Core
// QOpenGLFunctions_ES2
// QVulkanDeviceFunctions
// QVulkanFunctions
// QVulkanWindow
// QVulkanWindowRenderer

// ====== QtWidgets ======

// QAccessibleWidget
// QGraphicsBlurEffect
// QGraphicsColorizeEffect
// QGraphicsDropShadowEffect
// QGraphicsEffect
// QGraphicsOpacityEffect
// QGraphicsAnchor
// QGraphicsAnchorLayout
// QGraphicsGridLayout
// QAbstractGraphicsShapeItem
// QGraphicsEllipseItem
// QGraphicsItem
void QGraphicsItem_destructor(QGraphicsItem *graphicsItem);
void QGraphicsItem_delete(QGraphicsItem *graphicsItem);
// QGraphicsItemGroup
// QGraphicsLineItem
QGraphicsLineItem *QGraphicsLineItem_new();
void QGraphicsLineItem_destructor(QGraphicsLineItem *graphicsLineItem);
void QGraphicsLineItem_delete(QGraphicsLineItem *graphicsLineItem);
// QGraphicsObject
void QGraphicsObject_destructor(QGraphicsObject *graphicsObject);
void QGraphicsObject_delete(QGraphicsObject *graphicsObject);
// QGraphicsPathItem
// QGraphicsPixmapItem
// QGraphicsPolygonItem
// QGraphicsRectItem
// QGraphicsSimpleTextItem
// QGraphicsTextItem
// QGraphicsLayout
// QGraphicsLayoutItem
// QGraphicsLinearLayout
// QGraphicsProxyWidget
// QGraphicsScene
// QGraphicsSceneContextMenuEvent
// QGraphicsSceneDragDropEvent
// QGraphicsSceneEvent
// QGraphicsSceneHelpEvent
// QGraphicsSceneHoverEvent
// QGraphicsSceneMouseEvent
// QGraphicsSceneMoveEvent
// QGraphicsSceneResizeEvent
// QGraphicsSceneWheelEvent
// QGraphicsRotation
// QGraphicsScale
// QGraphicsTransform
// QGraphicsView
QGraphicsView *QGraphicsView_new();
void QGraphicsView_destructor(QGraphicsView *graphicsView);
void QGraphicsView_delete(QGraphicsView *graphicsView);
// QGraphicsWidget
QGraphicsWidget *QGraphicsWidget_new();
void QGraphicsWidget_destructor(QGraphicsWidget *graphicsWidget);
void QGraphicsWidget_delete(QGraphicsWidget *graphicsWidget);
// QAbstractItemDelegate
// QAbstractItemView
// QColumnView
// QDataWidgetMapper
// QFileIconProvider
// QHeaderView
// QItemDelegate
// QItemEditorCreator
// QItemEditorCreatorBase
// QItemEditorFactory
// QStandardItemEditorCreator
// QListView
QListView *QListView_new();
void QListView_destructor(QListView *listView);
void QListView_delete(QListView *listView);
// QListWidget
// QListWidgetItem
// QStyledItemDelegate
// QTableView
QTableView *QTableView_new();
void QTableView_destructor(QTableView *tableView);
void QTableView_delete(QTableView *tableView);
// QTableWidget
// QTableWidgetItem
// QTableWidgetSelectionRange
// QTreeView
QTreeView *QTreeView_new();
void QTreeView_destructor(QTreeView *treeView);
void QTreeView_delete(QTreeView *treeView);
// QTreeWidget
// QTreeWidgetItem
// QTreeWidgetItemIterator
// QAction
// QActionGroup
// QApplication
QApplication *QApplication_new(int *argc, char **argv);
// QBoxLayout
void QBoxLayout_destructor(QBoxLayout *boxLayout);
void QBoxLayout_delete(QBoxLayout *boxLayout);
void QBoxLayout_addStretch(QBoxLayout* boxLayout, int stretch);
void QBoxLayout_addSpacerItem(QBoxLayout* boxLayout, QSpacerItem *spacerItem);
void QBoxLayout_addWidget(QBoxLayout* boxLayout, QWidget *widget, int stretch, Qt::Alignment alignment);
void QBoxLayout_addLayout(QBoxLayout* boxLayout, QLayout *layout, int stretch);
void QBoxLayout_addStrut(QBoxLayout* boxLayout, int size);
void QBoxLayout_addItem(QBoxLayout* boxLayout, QLayoutItem * item);
void QBoxLayout_insertSpacing(QBoxLayout* boxLayout, int index, int size);
void QBoxLayout_insertStretch(QBoxLayout* boxLayout, int index, int stretch);
void QBoxLayout_insertSpacerItem(QBoxLayout* boxLayout, int index, QSpacerItem *spacerItem);
void QBoxLayout_insertWidget(QBoxLayout* boxLayout, int index, QWidget *widget, int stretch, Qt::Alignment alignment);
void QBoxLayout_insertLayout(QBoxLayout* boxLayout, int index, QLayout *layout, int stretch);
void QBoxLayout_insertItem(QBoxLayout* boxLayout, int index, QLayoutItem *item);
void QBoxLayout_setSpacing(QBoxLayout* boxLayout, int spacing);
bool QBoxLayout_setStretchFactor(QBoxLayout* boxLayout, QWidget *w, int stretch);
bool QBoxLayout_setStretchFactor1(QBoxLayout* boxLayout, QLayout *l, int stretch);
void QBoxLayout_setStretch(QBoxLayout* boxLayout, int index, int stretch);

// QHBoxLayout
QHBoxLayout *QHBoxLayout_new();
void QHBoxLayout_destructor(QHBoxLayout *hboxLayout);
void QHBoxLayout_delete(QHBoxLayout *hboxLayout);
// QVBoxLayout
QVBoxLayout *QVBoxLayout_new();
void QVBoxLayout_destructor(QVBoxLayout *vboxLayout);
void QVBoxLayout_delete(QVBoxLayout *vboxLayout);
// QFormLayout
QFormLayout *QFormLayout_new();
void QFormLayout_destructor(QFormLayout *formLayout);
void QFormLayout_delete(QFormLayout *formLayout);
void QFormLayout_setFieldGrowthPolicy(QFormLayout* formLayout, QFormLayout::FieldGrowthPolicy policy);
void QFormLayout_setRowWrapPolicy(QFormLayout* formLayout, QFormLayout::RowWrapPolicy policy);
void QFormLayout_setLabelAlignment(QFormLayout* formLayout, Qt::Alignment alignment);
void QFormLayout_setFormAlignment(QFormLayout* formLayout, Qt::Alignment alignment);
void QFormLayout_setHorizontalSpacing(QFormLayout* formLayout, int spacing);
void QFormLayout_setVerticalSpacing(QFormLayout* formLayout, int spacing);
void QFormLayout_setSpacing(QFormLayout* formLayout, int spacing);
void QFormLayout_addRow(QFormLayout* formLayout, QWidget *label, QWidget *field);
void QFormLayout_addRow1(QFormLayout* formLayout, QWidget *label, QLayout *field);
void QFormLayout_addRow2(QFormLayout* formLayout, const QString &labelText, QWidget *field);
void QFormLayout_addRow3(QFormLayout* formLayout, const QString &labelText, QLayout *field);
void QFormLayout_addRow4(QFormLayout* formLayout, QWidget *widget);
void QFormLayout_addRow5(QFormLayout* formLayout, QLayout *layout);
void QFormLayout_insertRow(QFormLayout* formLayout, int row, QWidget *label, QWidget *field);
void QFormLayout_insertRow1(QFormLayout* formLayout, int row, QWidget *label, QLayout *field);
void QFormLayout_insertRow2(QFormLayout* formLayout, int row, const QString &labelText, QWidget *field);
void QFormLayout_insertRow3(QFormLayout* formLayout, int row, const QString &labelText, QLayout *field);
void QFormLayout_insertRow4(QFormLayout* formLayout, int row, QWidget *widget);
void QFormLayout_insertRow5(QFormLayout* formLayout, int row, QLayout *layout);
void QFormLayout_removeRow(QFormLayout* formLayout, int row);
void QFormLayout_removeRow1(QFormLayout* formLayout, QWidget *widget);
void QFormLayout_removeRow2(QFormLayout* formLayout, QLayout *layout);
void QFormLayout_setItem(QFormLayout* formLayout, int row, QFormLayout::ItemRole role, QLayoutItem *item);
void QFormLayout_setWidget(QFormLayout* formLayout, int row, QFormLayout::ItemRole role, QWidget *widget);
void QFormLayout_setLayout(QFormLayout* formLayout, int row, QFormLayout::ItemRole role, QLayout *layout);
// QGesture
// QGestureEvent
// QPanGesture
// QPinchGesture
// QSwipeGesture
// QTapAndHoldGesture
// QTapGesture
// QGestureRecognizer
// QGridLayout
// QLayout
void QLayout_destructor(QLayout *layout);
void QLayout_delete(QLayout *layout);
// QLayoutItem
// QSpacerItem
// QWidgetItem
// QOpenGLWidget
// QShortcut
// QSizePolicy
// QStackedLayout
// QToolTip
// QWhatsThis
// QWidget
QWidget *QWidget_new();
void QWidget_destructor(QWidget *widget);
void QWidget_delete(QWidget *widget);
void QWidget_setStyle(QWidget* widget, QStyle *style);
void QWidget_setEnabled(QWidget* widget, bool enabled);
void QWidget_setDisabled(QWidget* widget, bool disabled);
void QWidget_setMinimumSize(QWidget* widget, int minw, int minh);
void QWidget_setMaximumSize(QWidget* widget, int maxw, int maxh);
void QWidget_setMinimumWidth(QWidget* widget, int minw);
void QWidget_setMinimumHeight(QWidget* widget, int minh);
void QWidget_setMaximumWidth(QWidget* widget, int maxw);
void QWidget_setMaximumHeight(QWidget* widget, int maxh);
void QWidget_setFixedSize(QWidget* widget, int w, int h);
void QWidget_setFixedWidth(QWidget* widget, int w);
void QWidget_setFixedHeight(QWidget* widget, int h);
void QWidget_mapToGlobal(const QWidget* widget, const QPoint &p, QPoint& out);
void QWidget_mapFromGlobal(const QWidget* widget, const QPoint &p, QPoint& out);
void QWidget_mapToParent(const QWidget* widget, const QPoint &p, QPoint& out);
void QWidget_mapFromParent(const QWidget* widget, const QPoint &p, QPoint& out);
void QWidget_mapTo(const QWidget* widget, const QWidget *other, const QPoint &p, QPoint& out);
void QWidget_mapFrom(const QWidget* widget, const QWidget *other, const QPoint &p, QPoint& out);
void QWidget_setFont(QWidget* widget, const QFont &font);
void QWidget_setCursor(QWidget* widget, const QCursor &cursor);
void QWidget_unsetCursor(QWidget* widget);
void QWidget_setMask(QWidget* widget, const QRegion &region);
void QWidget_clearMask(QWidget* widget);
void QWidget_setWindowIcon(QWidget* widget, const QIcon &icon);
void QWidget_setWindowIconText(QWidget* widget, const QString &windowIconText);
void QWidget_setToolTip(QWidget* widget, const QString &toolTip);
void QWidget_setToolTipDuration(QWidget* widget, int msec);
void QWidget_setStatusTip(QWidget* widget, const QString &statusTip);
void QWidget_setWhatsThis(QWidget* widget, const QString &whatsThis);
void QWidget_setAccessibleName(QWidget* widget, const QString &name);
void QWidget_setAccessibleDescription(QWidget* widget, const QString &description);
void QWidget_setLayoutDirection(QWidget* widget, Qt::LayoutDirection direction);
void QWidget_unsetLayoutDirection(QWidget* widget);
void QWidget_setLocale(QWidget* widget, const QLocale &locale);
void QWidget_unsetLocale(QWidget* widget);
bool QWidget_isActiveWindow(const QWidget* widget);
void QWidget_activateWindow(QWidget* widget);
void QWidget_clearFocus(QWidget* widget);
void QWidget_setFocus(QWidget* widget, Qt::FocusReason reason);
void QWidget_setFocusPolicy(QWidget* widget, Qt::FocusPolicy policy);
bool QWidget_hasFocus(const QWidget* widget);
void QWidget_setContextMenuPolicy(QWidget* widget, Qt::ContextMenuPolicy policy);
void QWidget_grabMouse(QWidget* widget);
void QWidget_grabMouse(QWidget* widget, const QCursor &cursor);
void QWidget_releaseMouse(QWidget* widget);
void QWidget_grabKeyboard(QWidget* widget);
void QWidget_releaseKeyboard(QWidget* widget);
int QWidget_grabShortcut(QWidget* widget, const QKeySequence &key, Qt::ShortcutContext context);
void QWidget_releaseShortcut(QWidget* widget, int id);
void QWidget_setShortcutEnabled(QWidget* widget, int id, bool enable);
void QWidget_setShortcutAutoRepeat(QWidget* widget, int id, bool enable);
void QWidget_update(QWidget* widget);
void QWidget_repaint(QWidget* widget);
void QWidget_setVisible(QWidget* widget, bool visible);
void QWidget_setHidden(QWidget* widget, bool hidden);
void QWidget_show(QWidget* widget);
void QWidget_hide(QWidget* widget);
void QWidget_showMinimized(QWidget* widget);
void QWidget_showMaximized(QWidget* widget);
void QWidget_showFullScreen(QWidget* widget);
void QWidget_showNormal(QWidget* widget);
bool QWidget_close(QWidget* widget);
void QWidget_raise(QWidget* widget);
void QWidget_lower(QWidget* widget);
void QWidget_stackUnder(QWidget* widget, QWidget* other);
void QWidget_move(QWidget* widget, int x, int y);
void QWidget_resize(QWidget* widget, int w, int h);
void QWidget_setGeometry(QWidget* widget, int x, int y, int w, int h);
void QWidget_setSizePolicy(QWidget* widget, QSizePolicy::Policy horizontal, QSizePolicy::Policy vertical);
void QWidget_setContentsMargins(QWidget* widget, int left, int top, int right, int bottom);
void QWidget_setLayout(QWidget* widget, QLayout *layout);
void QWidget_setParent(QWidget* widget, QWidget *parent);
QObject* QWidget_upcast_QObject(QWidget* self);
QPaintDevice* QWidget_upcast_QPaintDevice(QWidget* self);
// QWidgetAction
// QKeyEventTransition
// QMouseEventTransition
// QCommonStyle
// QTileRules
// QProxyStyle
// QStyle
// QStyleFactory
// QStyleHintReturn
// QStyleHintReturnMask
// QStyleHintReturnVariant
// QStyleOption
// QStyleOptionButton
// QStyleOptionComboBox
// QStyleOptionComplex
// QStyleOptionDockWidget
// QStyleOptionFocusRect
// QStyleOptionFrame
// QStyleOptionGraphicsItem
// QStyleOptionGroupBox
// QStyleOptionHeader
// QStyleOptionMenuItem
// QStyleOptionProgressBar
// QStyleOptionRubberBand
// QStyleOptionSizeGrip
// QStyleOptionSlider
// QStyleOptionSpinBox
// QStyleOptionTab
// QStyleOptionTabBarBase
// QStyleOptionTabWidgetFrame
// QStyleOptionTitleBar
// QStyleOptionToolBar
// QStyleOptionToolBox
// QStyleOptionToolButton
// QStyleOptionViewItem
// QStylePainter
// QStylePlugin
// QColormap
// QCompleter
// QScroller
// QScrollerProperties
// QSystemTrayIcon
// QUndoGroup
// QUndoCommand
// QUndoStack
// QUndoView
// QAbstractButton
void QAbstractButton_destructor(QAbstractButton *abstractButton);
void QAbstractButton_delete(QAbstractButton *abstractButton);
void QAbstractButton_destructor(QAbstractButton *abstractButton);
void QAbstractButton_delete(QAbstractButton *abstractButton);
void QAbstractButton_setText(QAbstractButton* abstractButton, const QString &text);
void QAbstractButton_setIcon(QAbstractButton* abstractButton, const QIcon &icon);
void QAbstractButton_setShortcut(QAbstractButton* abstractButton, const QKeySequence &key);
void QAbstractButton_setCheckable(QAbstractButton* abstractButton, bool checkable);
bool QAbstractButton_isChecked(const QAbstractButton* abstractButton);
void QAbstractButton_setDown(QAbstractButton* abstractButton, bool down);
bool QAbstractButton_isDown(const QAbstractButton* abstractButton);
void QAbstractButton_setAutoRepeat(QAbstractButton* abstractButton, bool autoRepeat);
void QAbstractButton_setAutoRepeatDelay(QAbstractButton* abstractButton, int autoRepeatDelay);
void QAbstractButton_setAutoRepeatInterval(QAbstractButton* abstractButton, int autoRepeatInterval);
void QAbstractButton_setAutoExclusive(QAbstractButton* abstractButton, bool autoExclusive);
void QAbstractButton_setIconSize(QAbstractButton* abstractButton, const QSize &size);
void QAbstractButton_animateClick(QAbstractButton* abstractButton, int msec);
void QAbstractButton_click(QAbstractButton* abstractButton);
void QAbstractButton_toggle(QAbstractButton* abstractButton);
void QAbstractButton_setChecked(QAbstractButton* abstractButton, bool checked);
// QAbstractScrollArea
QAbstractScrollArea *QAbstractScrollArea_new();
void QAbstractScrollArea_destructor(QAbstractScrollArea *abstractScrollArea);
void QAbstractScrollArea_delete(QAbstractScrollArea *abstractScrollArea);
// QAbstractSlider
QAbstractSlider *QAbstractSlider_new();
void QAbstractSlider_destructor(QAbstractSlider *abstractSlider);
void QAbstractSlider_delete(QAbstractSlider *abstractSlider);
// QAbstractSpinBox
// QButtonGroup
QButtonGroup *QButtonGroup_new();
void QButtonGroup_destructor(QButtonGroup *buttonGroup);
void QButtonGroup_delete(QButtonGroup *buttonGroup);
// QCalendarWidget
// QCheckBox
QCheckBox *QCheckBox_new();
void QCheckBox_destructor(QCheckBox *checkBox);
void QCheckBox_delete(QCheckBox *checkBox);
// QComboBox
QComboBox *QComboBox_new();
void QComboBox_destructor(QComboBox *comboBox);
void QComboBox_delete(QComboBox *comboBox);
void QComboBox_setMaxVisibleItems(QComboBox* comboBox, int maxItems);
int QComboBox_count(const QComboBox* comboBox);
void QComboBox_setMaxCount(QComboBox* comboBox, int max);
void QComboBox_setFrame(QComboBox* comboBox, bool frame);
void QComboBox_setInsertPolicy(QComboBox* comboBox, QComboBox::InsertPolicy policy);
void QComboBox_setSizeAdjustPolicy(QComboBox* comboBox, QComboBox::SizeAdjustPolicy policy);
void QComboBox_setMinimumContentsLength(QComboBox* comboBox, int characters);
void QComboBox_setIconSize(QComboBox* comboBox, const QSize &size);
void QComboBox_setEditable(QComboBox* comboBox, bool editable);
int QComboBox_currentIndex(const QComboBox* comboBox);
void QComboBox_addItem(QComboBox* comboBox, const QString &text, const QVariant &userData);
void QComboBox_addItem1(QComboBox* comboBox, const QIcon &icon, const QString &text, const QVariant &userData);
void QComboBox_insertItem(QComboBox* comboBox, int index, const QString &text, const QVariant &userData);
void QComboBox_insertItem1(QComboBox* comboBox, int index, const QIcon &icon, const QString &text, const QVariant &userData);
void QComboBox_insertSeparator(QComboBox* comboBox, int index);
void QComboBox_removeItem(QComboBox* comboBox, int index);
void QComboBox_setItemText(QComboBox* comboBox, int index, const QString &text);
void QComboBox_setItemIcon(QComboBox* comboBox, int index, const QIcon &icon);
void QComboBox_setItemData(QComboBox* comboBox, int index, const QVariant &value, int role);
void QComboBox_clear(QComboBox* comboBox);
void QComboBox_clearEditText(QComboBox* comboBox);
void QComboBox_setEditText(QComboBox* comboBox, const QString &text);
void QComboBox_setCurrentIndex(QComboBox* comboBox, int index);
void QComboBox_setCurrentText(QComboBox* comboBox, const QString &text);
// QCommandLinkButton
// QDateEdit
QDateEdit *QDateEdit_new();
void QDateEdit_destructor(QDateEdit *dateEdit);
void QDateEdit_delete(QDateEdit *dateEdit);
// QDateTimeEdit
QDateTimeEdit *QDateTimeEdit_new();
void QDateTimeEdit_destructor(QDateTimeEdit *dateTimeEdit);
void QDateTimeEdit_delete(QDateTimeEdit *dateTimeEdit);
// QTimeEdit
QTimeEdit *QTimeEdit_new();
void QTimeEdit_destructor(QTimeEdit *timeEdit);
void QTimeEdit_delete(QTimeEdit *timeEdit);
// QDial
// QDialogButtonBox
// QDockWidget
QDockWidget *QDockWidget_new();
void QDockWidget_destructor(QDockWidget *dockWidget);
void QDockWidget_delete(QDockWidget *dockWidget);
// QFocusFrame
// QFontComboBox
QFontComboBox *QFontComboBox_new();
void QFontComboBox_destructor(QFontComboBox *fontComboBox);
void QFontComboBox_delete(QFontComboBox *fontComboBox);
// QFrame
QFrame *QFrame_new();
void QFrame_destructor(QFrame *frame);
void QFrame_delete(QFrame *frame);
// QGroupBox
QGroupBox *QGroupBox_new();
void QGroupBox_destructor(QGroupBox *groupBox);
void QGroupBox_delete(QGroupBox *groupBox);
// QKeySequenceEdit
// QLabel
QLabel *QLabel_new();
void QLabel_destructor(QLabel *label);
void QLabel_delete(QLabel *label);
void QLabel_text(const QLabel* label, QString& out);
void QLabel_setTextFormat(QLabel* label, Qt::TextFormat);
void QLabel_setAlignment(QLabel* label, Qt::Alignment);
void QLabel_setWordWrap(QLabel* label, bool on);
void QLabel_setIndent(QLabel* label, int indent);
void QLabel_setMargin(QLabel* label, int margin);
void QLabel_setScaledContents(QLabel* label, bool scaledContents);
void QLabel_setBuddy(QLabel* label, QWidget *buddy);
void QLabel_setOpenExternalLinks(QLabel* label, bool open);
void QLabel_setTextInteractionFlags(QLabel* label, Qt::TextInteractionFlags flags);
void QLabel_setSelection(QLabel* label, int start, int length);
bool QLabel_hasSelectedText(const QLabel* label);
void QLabel_selectedText(const QLabel* label, QString& out);
int  QLabel_selectionStart(const QLabel* label);
void QLabel_setText(QLabel* label, const QString &text);
void QLabel_setPixmap(QLabel* label, const QPixmap &pixmap);
void QLabel_clear(QLabel* label);
// QLCDNumber
// QLineEdit
QLineEdit *QLineEdit_new();
void QLineEdit_destructor(QLineEdit *lineEdit);
void QLineEdit_delete(QLineEdit *lineEdit);
void QLineEdit_text(const QLineEdit* lineEdit, QString& out);
void QLineEdit_displayText(const QLineEdit* lineEdit, QString& out);
void QLineEdit_setPlaceholderText(QLineEdit* lineEdit, const QString &placeholder);
void QLineEdit_setMaxLength(QLineEdit* lineEdit, int maxLength);
void QLineEdit_setFrame(QLineEdit* lineEdit, bool frame);
void QLineEdit_setClearButtonEnabled(QLineEdit* lineEdit, bool enable);
void QLineEdit_setEchoMode(QLineEdit* lineEdit, QLineEdit::EchoMode echoMode);
void QLineEdit_setReadOnly(QLineEdit* lineEdit, bool readonly);
int QLineEdit_cursorPosition(const QLineEdit* lineEdit);
void QLineEdit_setCursorPosition(QLineEdit* lineEdit, int pos);
int QLineEdit_cursorPositionAt(QLineEdit* lineEdit, const QPoint &pos);
void QLineEdit_setAlignment(QLineEdit* lineEdit, Qt::Alignment alignment);
void QLineEdit_cursorForward(QLineEdit* lineEdit, bool mark, int steps);
void QLineEdit_cursorBackward(QLineEdit* lineEdit, bool mark, int steps);
void QLineEdit_cursorWordForward(QLineEdit* lineEdit, bool mark);
void QLineEdit_cursorWordBackward(QLineEdit* lineEdit, bool mark);
void QLineEdit_backspace(QLineEdit* lineEdit);
void QLineEdit_del(QLineEdit* lineEdit);
void QLineEdit_home(QLineEdit* lineEdit, bool mark);
void QLineEdit_end(QLineEdit* lineEdit, bool mark);
bool QLineEdit_isModified(const QLineEdit* lineEdit);
void QLineEdit_setModified(QLineEdit* lineEdit, bool modified);
void QLineEdit_setSelection(QLineEdit* lineEdit, int start, int length);
bool QLineEdit_hasSelectedText(QLineEdit* lineEdit);
void QLineEdit_selectedText(const QLineEdit* lineEdit, QString& out);
int QLineEdit_selectionStart(const QLineEdit* lineEdit);
int QLineEdit_selectionEnd(const QLineEdit* lineEdit);
int QLineEdit_selectionLength(const QLineEdit* lineEdit);
bool QLineEdit_isUndoAvailable(const QLineEdit* lineEdit);
bool QLineEdit_isRedoAvailable(const QLineEdit* lineEdit);
void QLineEdit_setDragEnabled(QLineEdit* lineEdit, bool dragEnabled);
void QLineEdit_setCursorMoveStyle(QLineEdit* lineEdit, Qt::CursorMoveStyle style);
void QLineEdit_setInputMask(QLineEdit* lineEdit, const QString &inputMask);
bool QLineEdit_hasAcceptableInput(const QLineEdit* lineEdit);
void QLineEdit_setTextMargins(QLineEdit* lineEdit, int left, int top, int right, int bottom);
void QLineEdit_setText(QLineEdit* lineEdit, const QString &text);
void QLineEdit_clear(QLineEdit* lineEdit);
void QLineEdit_selectAll(QLineEdit* lineEdit);
void QLineEdit_undo(QLineEdit* lineEdit);
void QLineEdit_redo(QLineEdit* lineEdit);
void QLineEdit_cut(QLineEdit* lineEdit);
void QLineEdit_copy(const QLineEdit* lineEdit);
void QLineEdit_paste(QLineEdit* lineEdit);
void QLineEdit_deselect(QLineEdit* lineEdit);
void QLineEdit_insert(QLineEdit* lineEdit, const QString &text);
QMenu *QLineEdit_createStandardContextMenu(QLineEdit* lineEdit);
// QMainWindow
// QMdiArea
// QMdiSubWindow
// QMenu
QMenu *QMenu_new();
void QMenu_destructor(QMenu *menu);
void QMenu_delete(QMenu *menu);
// QMenuBar
QMenuBar *QMenuBar_new();
void QMenuBar_destructor(QMenuBar *menuBar);
void QMenuBar_delete(QMenuBar *menuBar);
// QPlainTextDocumentLayout
// QPlainTextEdit
QPlainTextEdit *QPlainTextEdit_new();
void QPlainTextEdit_destructor(QPlainTextEdit *plainTextEdit);
void QPlainTextEdit_delete(QPlainTextEdit *plainTextEdit);
// QProgressBar
QProgressBar *QProgressBar_new();
void QProgressBar_destructor(QProgressBar *progressBar);
void QProgressBar_delete(QProgressBar *progressBar);
// QPushButton
QPushButton *QPushButton_new();
void QPushButton_destructor(QPushButton *pushButton);
void QPushButton_delete(QPushButton *pushButton);
void QPushButton_setAutoDefault(QPushButton* pushButton, bool autoDefault);
void QPushButton_setDefault(QPushButton* pushButton, bool default_);
void QPushButton_setMenu(QPushButton* pushButton, QMenu* menu);
void QPushButton_setFlat(QPushButton* pushButton, bool flat);
void QPushButton_showMenu(QPushButton* pushButton);
// QRadioButton
QRadioButton *QRadioButton_new();
void QRadioButton_destructor(QRadioButton *radioButton);
void QRadioButton_delete(QRadioButton *radioButton);
// QRubberBand
// QScrollArea
QScrollArea *QScrollArea_new();
void QScrollArea_destructor(QScrollArea *scrollArea);
void QScrollArea_delete(QScrollArea *scrollArea);
// QScrollBar
QScrollBar *QScrollBar_new();
void QScrollBar_destructor(QScrollBar *scrollBar);
void QScrollBar_delete(QScrollBar *scrollBar);
// QSizeGrip
// QSlider
QSlider *QSlider_new();
void QSlider_destructor(QSlider *slider);
void QSlider_delete(QSlider *slider);
// QDoubleSpinBox
QDoubleSpinBox *QDoubleSpinBox_new();
void QDoubleSpinBox_destructor(QDoubleSpinBox *doubleSpinBox);
void QDoubleSpinBox_delete(QDoubleSpinBox *doubleSpinBox);
// QSpinBox
QSpinBox *QSpinBox_new();
void QSpinBox_destructor(QSpinBox *spinBox);
void QSpinBox_delete(QSpinBox *spinBox);
// QSplashScreen
// QSplitter
// QSplitterHandle
// QStackedWidget
// QStatusBar
QStatusBar *QStatusBar_new();
void QStatusBar_destructor(QStatusBar *statusBar);
void QStatusBar_delete(QStatusBar *statusBar);
// QTabBar
// QTabWidget
// QTextBrowser
// QTextEdit
QTextEdit *QTextEdit_new();
void QTextEdit_destructor(QTextEdit *textEdit);
void QTextEdit_delete(QTextEdit *textEdit);
// QToolBar
// QToolBox
// QToolButton
// QColorDialog
// QDialog
// QErrorMessage
// QFileDialog
// QFileSystemModel
// QFontDialog
// QInputDialog
// QMessageBox
// QProgressDialog
// QWizard
// QWizardPage
// QMacCocoaViewContainer
// QMacNativeWidget

// ====== miniqt ======

using MQCallback_ptr = void (*)(uintptr_t, uintptr_t);
using MQCallback_QString_ptr = void (*)(uintptr_t, uintptr_t, const QString &);
using MQCallback_int_ptr = void (*)(uintptr_t, uintptr_t, int);

QObject *MQCallback_new(uintptr_t data0, uintptr_t data1,
                        MQCallback_ptr callback);
QObject *MQCallback_int_new(uintptr_t data0, uintptr_t data1,
                            MQCallback_int_ptr callback);
QObject *MQCallback_QString_new(uintptr_t data0, uintptr_t data1,
                                MQCallback_QString_ptr callback);


using MQPaintEventCallback = bool(*)(uintptr_t, uintptr_t, QWidget*, const QPaintEvent&);
class MQPaintEventFilter : public QObject
{
public:
	MQPaintEventFilter(uintptr_t data0, uintptr_t data1, MQPaintEventCallback callback);
	~MQPaintEventFilter();
	bool eventFilter(QObject *receiver, QEvent* event) override;

private:
	uintptr_t data0_;
	uintptr_t data1_;
	MQPaintEventCallback callback_;
};

void MQPaintEventFilter_constructor(MQPaintEventFilter* self, uintptr_t data0, uintptr_t data1, MQPaintEventCallback callback); 
void MQPaintEventFilter_destructor(MQPaintEventFilter* self);

//QWidget *MQCustomWidget_new();
//void MQCustomWidget_delete(QWidget* widget);

#endif // MINIQT_H
