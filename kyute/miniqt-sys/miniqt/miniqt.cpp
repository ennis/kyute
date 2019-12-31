#include "miniqt.hpp"

//=============================================================================
QApplication *QApplication_new(int *argc, char **argv) {
  auto d = new QApplication(*argc, argv);
  QApplication::setStyle(QStyleFactory::create("Fusion"));
  // modify palette to dark
  QPalette darkPalette;
  darkPalette.setColor(QPalette::Window, QColor(53, 53, 53));
  darkPalette.setColor(QPalette::WindowText, Qt::white);
  darkPalette.setColor(QPalette::Disabled, QPalette::WindowText,
                       QColor(127, 127, 127));
  darkPalette.setColor(QPalette::Base, QColor(42, 42, 42));
  darkPalette.setColor(QPalette::AlternateBase, QColor(66, 66, 66));
  darkPalette.setColor(QPalette::ToolTipBase, Qt::white);
  darkPalette.setColor(QPalette::ToolTipText, Qt::white);
  darkPalette.setColor(QPalette::Text, Qt::white);
  darkPalette.setColor(QPalette::Disabled, QPalette::Text,
                       QColor(127, 127, 127));
  darkPalette.setColor(QPalette::Dark, QColor(35, 35, 35));
  darkPalette.setColor(QPalette::Shadow, QColor(20, 20, 20));
  darkPalette.setColor(QPalette::Button, QColor(53, 53, 53));
  darkPalette.setColor(QPalette::ButtonText, Qt::white);
  darkPalette.setColor(QPalette::Disabled, QPalette::ButtonText,
                       QColor(127, 127, 127));
  darkPalette.setColor(QPalette::BrightText, Qt::red);
  darkPalette.setColor(QPalette::Link, QColor(42, 130, 218));
  darkPalette.setColor(QPalette::Highlight, QColor(42, 130, 218));
  darkPalette.setColor(QPalette::Disabled, QPalette::Highlight,
                       QColor(80, 80, 80));
  darkPalette.setColor(QPalette::HighlightedText, Qt::white);
  darkPalette.setColor(QPalette::Disabled, QPalette::HighlightedText,
                       QColor(127, 127, 127));
  QApplication::setPalette(darkPalette);
  QFont font;
  font.setPointSizeF(11.0);
  QApplication::setFont(font);
  return d;
}

//=============================================================================
void QCoreApplication_processEvents(QEventLoop::ProcessEventsFlags flags) {
  QCoreApplication::processEvents(flags);
}
void QCoreApplication_installEventFilter(QObject *filterObj) {
  QCoreApplication::instance()->installEventFilter(filterObj);
}
void QCoreApplication_removeEventFilter(QObject *filterObj) {
  QCoreApplication::instance()->removeEventFilter(filterObj);
}

//=============================================================================
QEventLoop* QEventLoop_new() {
	return new QEventLoop();
}

void QEventLoop_destructor(QEventLoop* eventLoop) {
	eventLoop->~QEventLoop();
}

void QEventLoop_delete(QEventLoop* eventLoop) {
	delete eventLoop;
}

void QEventLoop_processEvents(QEventLoop* eventLoop, QEventLoop::ProcessEventsFlags flags) {
	eventLoop->processEvents(flags);
}

bool QEventLoop_isRunning(const QEventLoop* eventLoop) {
	return eventLoop->isRunning();
}

//=============================================================================
QByteArray *QByteArray_new() { return new QByteArray(); }
void QByteArray_destructor(QByteArray *byteArray) { byteArray->~QByteArray(); }
void QByteArray_delete(QByteArray *byteArray) { delete byteArray; }

//=============================================================================
void QObject_destructor(QObject *object) { object->~QObject(); }
void QObject_delete(QObject *object) { delete object; }
void QObject_connect_abi(const QObject *sender, const char *signal,
                         const QObject *receiver, const char *method,
                         Qt::ConnectionType type) {
  QObject::connect(sender, signal, receiver, method, type);
}
void QObject_installEventFilter(QObject *self, QObject *filterObj) {
  self->installEventFilter(filterObj);
}
void QObject_removeEventFilter(QObject *self, QObject *filterObj) {
  self->removeEventFilter(filterObj);
}
bool QObject_setProperty(QObject *self, const char *name,
                         const QVariant &value) {
  return self->setProperty(name, value);
}
void QObject_property(const QObject *self, const char *name,
                      QVariant &outVariant) {
  outVariant = self->property(name);
}
bool QObject_property_uint64(const QObject *self, const char *name,
                             uint64_t &outValue) {
  QVariant v = self->property(name);
  bool ok = false;
  outValue = v.toULongLong(&ok);
  return ok;
}
bool QObject_setProperty_uint64(QObject *self, const char *name,
                                uint64_t value) {
  return self->setProperty(name, QVariant(value));
}
QWidget *QObject_downcast_QWidget(QObject *self) {
  return qobject_cast<QWidget *>(self);
}

//=============================================================================
void QRect_getCoords(const QRect* rect, int *x1, int *y1, int *x2, int *y2) {
	rect->getCoords(x1, y1, x2, y2);
}

//=============================================================================
void QRectF_constructor(QRectF* rect, qreal x, qreal y, qreal w, qreal h) {
	new (rect) QRectF(x, y, w, h);
}

//=============================================================================
void QString_constructor(QString *string) { new (string) QString(); }
void QString_destructor(QString *string) { string->~QString(); }
int QString_size(const QString *string) { return string->size(); }
const uint16_t *QString_utf16(const QString *string) { return string->utf16(); }
void QString_fromUtf8(const char *str, int size, QString &out) {
  out = QString::fromUtf8(str, size);
}

//=============================================================================
void QStringList_destructor(QStringList *stringList) {
  stringList->~QStringList();
}
//=============================================================================
void QVariant_constructor_quint64(QVariant *variant, quint64 v) {
  new (variant) QVariant(v);
}
void QVariant_destructor(QVariant *variant) { variant->~QVariant(); }

//=============================================================================
void QPixmap_destructor(QPixmap *pixmap) { pixmap->~QPixmap(); }

//=============================================================================
const QRect& QPaintEvent_rect(const QPaintEvent* paintEvent) {
	return paintEvent->rect();
}

//=============================================================================
void QBrush_destructor(QBrush *brush) { brush->~QBrush(); }
void QBrush_constructor(QBrush *brush) { new (brush) QBrush; }
void QBrush_constructor1(QBrush *brush, const QColor &color) {
  new (brush) QBrush(color);
}
void QBrush_constructor2(QBrush *brush, const QGradient &gradient) {
  new (brush) QBrush(gradient);
}

//=============================================================================
void QConicalGradient_destructor(QConicalGradient *conicalGradient) {
  conicalGradient->~QConicalGradient();
}

//=============================================================================
void QGradient_constructor(QGradient *gradient) { new (gradient) QGradient; }
void QGradient_destructor(QGradient *gradient) { gradient->~QGradient(); }
void QGradient_setSpread(QGradient *gradient, QGradient::Spread spread) {
  gradient->setSpread(spread);
}
QGradient::Spread QGradient_spread(const QGradient *gradient) {
  return gradient->spread();
}
void QGradient_setColorAt(QGradient *gradient, qreal pos, const QColor &color) {
  gradient->setColorAt(pos, color);
}
QGradient::CoordinateMode QGradient_coordinateMode(const QGradient *gradient) {
  return gradient->coordinateMode();
}
void QGradient_setCoordinateMode(QGradient *gradient,
                                 QGradient::CoordinateMode mode) {
  gradient->setCoordinateMode(mode);
}
QGradient::InterpolationMode
QGradient_interpolationMode(const QGradient *gradient) {
  return gradient->interpolationMode();
}
void QGradient_setInterpolationMode(QGradient *gradient,
                                    QGradient::InterpolationMode mode) {
  gradient->setInterpolationMode(mode);
}

//=============================================================================
void QLinearGradient_destructor(QLinearGradient *linearGradient) {
  linearGradient->~QLinearGradient();
}
void QLinearGradient_constructor(QLinearGradient *linearGradient) {
  new (linearGradient) QLinearGradient;
}
void QLinearGradient_constructor1(QLinearGradient *linearGradient,
                                  const QPointF &start,
                                  const QPointF &finalStop) {
  new (linearGradient) QLinearGradient(start, finalStop);
}

//=============================================================================
void QRadialGradient_destructor(QRadialGradient *radialGradient) {
  radialGradient->~QRadialGradient();
}
void QRadialGradient_constructor(QRadialGradient *radialGradient) {
  new (radialGradient) QRadialGradient;
}

//=============================================================================
void QColor_constructor(QColor *color) { new (color) QColor(); }
void QColor_destructor(QColor *color) { color->~QColor(); }

void QColor_fromRgb(QColor *color, int r, int g, int b, int a) {
  *color = QColor::fromRgb(r, g, b, a);
}
void QColor_fromRgbF(QColor *color, qreal r, qreal g, qreal b, qreal a) {
  *color = QColor::fromRgbF(r, g, b, a);
}
void QColor_fromHsv(QColor *color, int h, int s, int v, int a) {
  *color = QColor::fromHsv(h, s, v, a);
}
void QColor_fromHsvF(QColor *color, qreal h, qreal s, qreal v, qreal a) {
  *color = QColor::fromHsvF(h, s, v, a);
}
void QColor_fromCmyk(QColor *color, int c, int m, int y, int k, int a) {
  *color = QColor::fromCmyk(c, m, y, k, a);
}
void QColor_fromCmykF(QColor *color, qreal c, qreal m, qreal y, qreal k,
                      qreal a) {
  *color = QColor::fromCmykF(c, m, y, k, a);
}
void QColor_fromHsl(QColor *color, int h, int s, int l, int a) {
  *color = QColor::fromHsl(h, s, l, a);
}
void QColor_fromHslF(QColor *color, qreal h, qreal s, qreal l, qreal a) {
  *color = QColor::fromHslF(h, s, l, a);
}
void QColor_fromRgba64(QColor *color, ushort r, ushort g, ushort b, ushort a) {
	*color = QColor::fromRgba64(r, g, b, a);
}
qreal QColor_redF(const QColor* color) { return color->redF(); }
qreal QColor_greenF(const QColor* color) { return color->greenF(); }
qreal QColor_blueF(const QColor* color) { return color->blueF(); }
qreal QColor_alphaF(const QColor* color) { return color->alphaF(); }

//=============================================================================
QWidget *QPaintDevice_downcast_QWidget(QPaintDevice *paintDevice) {
  return static_cast<QWidget *>(paintDevice);
}

//=============================================================================
QPainter *QPainter_new() { return new QPainter(); }
void QPainter_constructor(QPainter *painter) { new (painter) QPainter; }
void QPainter_constructor1(QPainter *painter, QPaintDevice *paintDevice) {
  new (painter) QPainter(paintDevice);
}
void QPainter_destructor(QPainter *painter) { painter->~QPainter(); }
void QPainter_delete(QPainter *painter) { delete painter; }
void QPainter_setCompositionMode(QPainter *painter,
                                 QPainter::CompositionMode mode) {
  painter->setCompositionMode(mode);
}
void QPainter_setFont(QPainter *painter, const QFont &f) {
  painter->setFont(f);
}
void QPainter_setPen(QPainter *painter, const QColor &color) {
  painter->setPen(color);
}
void QPainter_setPen1(QPainter *painter, const QPen &pen) {
  painter->setPen(pen);
}
void QPainter_setPen2(QPainter *painter, Qt::PenStyle style) {
  painter->setPen(style);
}
void QPainter_setBrush(QPainter *painter, const QBrush &brush) {
  painter->setBrush(brush);
}
void QPainter_setBrush1(QPainter *painter, Qt::BrushStyle style) {
  painter->setBrush(style);
}
void QPainter_setBackgroundMode(QPainter *painter, Qt::BGMode mode) {
  painter->setBackgroundMode(mode);
}
void QPainter_setBrushOrigin(QPainter *painter, const QPointF &origin) {
  painter->setBrushOrigin(origin);
}
void QPainter_setBackground(QPainter *painter, const QBrush &bg) {
  painter->setBackground(bg);
}
void QPainter_setOpacity(QPainter *painter, qreal opacity) {
  painter->setOpacity(opacity);
}
void QPainter_setClipRect(QPainter *painter, const QRectF &rect,
                          Qt::ClipOperation op) {
  painter->setClipRect(rect, op);
}
void QPainter_setClipRegion(QPainter *painter, const QRegion &region,
                            Qt::ClipOperation op) {
  painter->setClipRegion(region, op);
}
void QPainter_setClipPath(QPainter *painter, const QPainterPath &path,
                          Qt::ClipOperation op) {
  painter->setClipPath(path, op);
}
void QPainter_setClipping(QPainter *painter, bool enable) {
  painter->setClipping(enable);
}
bool QPainter_hasClipping(const QPainter *painter) {
  return painter->hasClipping();
}
void QPainter_save(QPainter *painter) { painter->save(); }
void QPainter_restore(QPainter *painter) { painter->restore(); }
void QPainter_setTransform(QPainter *painter, const QTransform &transform,
                           bool combine) {
  painter->setTransform(transform, combine);
}
void QPainter_resetTransform(QPainter *painter) { painter->resetTransform(); }
void QPainter_setWorldTransform(QPainter *painter, const QTransform &matrix,
                                bool combine) {
  painter->setWorldTransform(matrix, combine);
}
void QPainter_setWorldMatrixEnabled(QPainter *painter, bool enabled) {
  painter->setWorldMatrixEnabled(enabled);
}
void QPainter_scale(QPainter *painter, qreal sx, qreal sy) {
  painter->scale(sx, sy);
}
void QPainter_shear(QPainter *painter, qreal sh, qreal sv) {
  painter->shear(sh, sv);
}
void QPainter_rotate(QPainter *painter, qreal a) { painter->rotate(a); }
void QPainter_translate(QPainter *painter, qreal dx, qreal dy) {
  painter->translate(dx, dy);
}
void QPainter_setWindow(QPainter *painter, const QRect &rect) {
  painter->setWindow(rect);
}
void QPainter_setViewport(QPainter *painter, const QRect &rect) {
  painter->setViewport(rect);
}
void QPainter_setViewTransformEnabled(QPainter *painter, bool enable) {
  painter->setViewTransformEnabled(enable);
}
void QPainter_strokePath(QPainter *painter, const QPainterPath &path,
                         const QPen &pen) {
  painter->strokePath(path, pen);
}
void QPainter_fillPath(QPainter *painter, const QPainterPath &path,
                       const QBrush &brush) {
  painter->fillPath(path, brush);
}
void QPainter_drawPath(QPainter *painter, const QPainterPath &path) {
  painter->drawPath(path);
}
void QPainter_drawPoint(QPainter *painter, const QPointF &p) {
  painter->drawPoint(p);
}
void QPainter_drawLine(QPainter *painter, const QPointF &start,
                       const QPointF &end) {
  painter->drawLine(start, end);
}
void QPainter_drawRect(QPainter *painter, const QRectF &rect) {
  painter->drawRect(rect);
}
void QPainter_drawEllipse(QPainter *painter, const QRectF &rect) {
  painter->drawEllipse(rect);
}
void QPainter_drawEllipse1(QPainter *painter, const QPointF &center, qreal rx,
                           qreal ry) {
  painter->drawEllipse(center, rx, ry);
}
void QPainter_drawPolyline(QPainter *painter, const QPointF *points,
                           int pointCount) {
  painter->drawPolyline(points, pointCount);
}
void QPainter_drawPolygon(QPainter *painter, const QPointF *points,
                          int pointCount, Qt::FillRule fillRule) {
  painter->drawPolygon(points, pointCount, fillRule);
}
void QPainter_drawConvexPolygon(QPainter *painter, const QPointF *points,
                                int pointCount) {
  painter->drawConvexPolygon(points, pointCount);
}
void QPainter_drawArc(QPainter *painter, const QRectF &rect, int a, int alen) {
  painter->drawArc(rect, a, alen);
}
void QPainter_drawPie(QPainter *painter, const QRectF &rect, int a, int alen) {
  painter->drawPie(rect, a, alen);
}
void QPainter_drawChord(QPainter *painter, const QRectF &rect, int a,
                        int alen) {
  painter->drawChord(rect, a, alen);
}
void QPainter_drawRoundedRect(QPainter *painter, const QRectF &rect,
                              qreal xRadius, qreal yRadius, Qt::SizeMode mode) {
  painter->drawRoundedRect(rect, xRadius, yRadius, mode);
}
void QPainter_drawTiledPixmap(QPainter *painter, const QRectF &rect,
                              const QPixmap &pm, const QPointF &p) {
  painter->drawTiledPixmap(rect, pm, p);
}
void QPainter_drawPixmap(QPainter *painter, const QRectF &dst,
                         const QPixmap &pixmap, const QRectF &src) {
  painter->drawPixmap(dst, pixmap, src);
}
void QPainter_drawPixmap1(QPainter *painter, const QPointF &dst,
                          const QPixmap &pm) {
  painter->drawPixmap(dst, pm);
}
void QPainter_drawPixmapFragments(QPainter *painter,
                                  const QPainter::PixmapFragment *fragments,
                                  int fragmentCount, const QPixmap &pixmap,
                                  QPainter::PixmapFragmentHints hints) {
  painter->drawPixmapFragments(fragments, fragmentCount, pixmap, hints);
}
void QPainter_drawImage(QPainter *painter, const QRectF &dst,
                        const QImage &image, const QRectF &src,
                        Qt::ImageConversionFlags flags) {
  painter->drawImage(dst, image, src, flags);
}
void QPainter_drawImage1(QPainter *painter, const QPointF &dst,
                         const QImage &image) {
  painter->drawImage(dst, image);
}
void QPainter_setLayoutDirection(QPainter *painter,
                                 Qt::LayoutDirection direction) {
  painter->setLayoutDirection(direction);
}
void QPainter_drawGlyphRun(QPainter *painter, const QPointF &pos,
                           const QGlyphRun &glyphRun) {
  painter->drawGlyphRun(pos, glyphRun);
}
void QPainter_drawStaticText(QPainter *painter, const QPointF &pos,
                             const QStaticText &staticText) {
  painter->drawStaticText(pos, staticText);
}
void QPainter_drawText(QPainter *painter, const QPointF &pos,
                       const QString &s) {
  painter->drawText(pos, s);
}
void QPainter_drawText1(QPainter *painter, const QRectF &rect,
                        const QString &text, const QTextOption &o) {
  painter->drawText(rect, text, o);
}
void QPainter_boundingRect(QPainter *painter, const QRectF &rect,
                           const QString &text, const QTextOption &o,
                           QRectF &out) {
  out = painter->boundingRect(rect, text, o);
}
void QPainter_setRenderHint(QPainter *painter, QPainter::RenderHint hint,
                            bool on) {
  painter->setRenderHint(hint, on);
}
void QPainter_setRenderHints(QPainter *painter, QPainter::RenderHints hints,
                             bool on) {
  painter->setRenderHints(hints, on);
}
void QPainter_beginNativePainting(QPainter *painter) {
  painter->beginNativePainting();
}
void QPainter_endNativePainting(QPainter *painter) {
  painter->endNativePainting();
}

//=============================================================================
QPainterPath *QPainterPath_new() { return new QPainterPath(); }
void QPainterPath_constructor(QPainterPath *painterPath) {
  new (painterPath) QPainterPath();
}
void QPainterPath_destructor(QPainterPath *painterPath) {
  painterPath->~QPainterPath();
}
void QPainterPath_delete(QPainterPath *painterPath) { delete painterPath; }
void QPainterPath_addEllipse(QPainterPath *painterPath, qreal x, qreal y,
                             qreal width, qreal height) {
  painterPath->addEllipse(x, y, width, height);
}
void QPainterPath_addPath(QPainterPath *painterPath, const QPainterPath &path) {
  painterPath->addPath(path);
}
void QPainterPath_addPolygon(QPainterPath *painterPath,
                             const QPolygonF &polygon) {
  painterPath->addPolygon(polygon);
}
void QPainterPath_addRect(QPainterPath *painterPath, qreal x, qreal y,
                          qreal width, qreal height) {
  painterPath->addRect(x, y, width, height);
}
void QPainterPath_addRegion(QPainterPath *painterPath, const QRegion &region) {
  painterPath->addRegion(region);
}
void QPainterPath_addRoundedRect(QPainterPath *painterPath, qreal x, qreal y,
                                 qreal w, qreal h, qreal xRadius, qreal yRadius,
                                 Qt::SizeMode mode) {
  painterPath->addRoundedRect(x, y, w, h, xRadius, yRadius, mode);
}
void QPainterPath_addText(QPainterPath *painterPath, qreal x, qreal y,
                          const QFont &font, const QString &text) {
  painterPath->addText(x, y, font, text);
}
qreal QPainterPath_angleAtPercent(QPainterPath *painterPath, qreal t) {
  return painterPath->angleAtPercent(t);
}
void QPainterPath_arcMoveTo(QPainterPath *painterPath, qreal x, qreal y,
                            qreal width, qreal height, qreal angle) {
  painterPath->arcMoveTo(x, y, width, height, angle);
}
void QPainterPath_arcTo(QPainterPath *painterPath, qreal x, qreal y,
                        qreal width, qreal height, qreal startAngle,
                        qreal sweepLength) {
  painterPath->arcTo(x, y, width, height, startAngle, sweepLength);
}
void QPainterPath_boundingRect(QPainterPath *painterPath, QRectF &out) {
  out = painterPath->boundingRect();
}
int QPainterPath_capacity(const QPainterPath *painterPath) {
  return painterPath->capacity();
}
void QPainterPath_clear(QPainterPath *painterPath) { painterPath->clear(); }
void QPainterPath_closeSubpath(QPainterPath *painterPath) {
  painterPath->closeSubpath();
}
void QPainterPath_connectPath(QPainterPath *painterPath,
                              const QPainterPath &path) {
  painterPath->connectPath(path);
}
bool QPainterPath_contains(const QPainterPath *painterPath,
                           const QPointF &point) {
  return painterPath->contains(point);
}
bool QPainterPath_contains(const QPainterPath *painterPath,
                           const QRectF &rectangle) {
  return painterPath->contains(rectangle);
}
bool QPainterPath_contains(const QPainterPath *painterPath,
                           const QPainterPath &p) {
  return painterPath->contains(p);
}
void QPainterPath_cubicTo(QPainterPath *painterPath, qreal c1X, qreal c1Y,
                          qreal c2X, qreal c2Y, qreal endPointX,
                          qreal endPointY) {
  painterPath->cubicTo(c1X, c1Y, c2X, c2Y, endPointX, endPointY);
}
int QPainterPath_elementCount(const QPainterPath *painterPath) {
  return painterPath->elementCount();
}
void QPainterPath_intersected(const QPainterPath *painterPath,
                              const QPainterPath &p, QPainterPath &out) {
  out = painterPath->intersected(p);
}
bool QPainterPath_intersects(const QPainterPath *painterPath,
                             const QRectF &rectangle) {
  return painterPath->intersects(rectangle);
}
bool QPainterPath_intersects(const QPainterPath *painterPath,
                             const QPainterPath &p) {
  return painterPath->intersects(p);
}
bool QPainterPath_isEmpty(const QPainterPath *painterPath) {
  return painterPath->isEmpty();
}
qreal QPainterPath_length(const QPainterPath *painterPath) {
  return painterPath->length();
}
void QPainterPath_lineTo(QPainterPath *painterPath, qreal x, qreal y) {
  painterPath->lineTo(x, y);
}
void QPainterPath_moveTo(QPainterPath *painterPath, qreal x, qreal y) {
  painterPath->moveTo(x, y);
}
qreal QPainterPath_percentAtLength(QPainterPath *painterPath, qreal len) {
  return painterPath->percentAtLength(len);
}
void QPainterPath_quadTo(QPainterPath *painterPath, qreal cx, qreal cy,
                         qreal endPointX, qreal endPointY) {
  painterPath->quadTo(cx, cy, endPointX, endPointY);
}
void QPainterPath_reserve(QPainterPath *painterPath, int size) {
  painterPath->reserve(size);
}
void QPainterPath_setElementPositionAt(QPainterPath *painterPath, int index,
                                       qreal x, qreal y) {
  painterPath->setElementPositionAt(index, x, y);
}
void QPainterPath_setFillRule(QPainterPath *painterPath,
                              Qt::FillRule fillRule) {
  painterPath->setFillRule(fillRule);
}
void QPainterPath_simplified(const QPainterPath *painterPath,
                             QPainterPath &out) {
  out = painterPath->simplified();
}
qreal QPainterPath_slopeAtPercent(const QPainterPath *painterPath, qreal t) {
  return painterPath->slopeAtPercent(t);
}
void QPainterPath_subtracted(const QPainterPath *painterPath,
                             const QPainterPath &p, QPainterPath &out) {
  out = painterPath->subtracted(p);
}
void QPainterPath_swap(QPainterPath *painterPath, QPainterPath &other) {
  painterPath->swap(other);
}
void QPainterPath_toFillPolygon1(QPainterPath *painterPath,
                                 const QMatrix &matrix, QPolygonF &out) {
  out = painterPath->toFillPolygon(matrix);
}
void QPainterPath_toReversed(const QPainterPath *painterPath,
                             QPainterPath &out) {
  out = painterPath->toReversed();
}
void QPainterPath_translate(QPainterPath *painterPath, qreal dx, qreal dy) {
  painterPath->translate(dx, dy);
}
void QPainterPath_translated(const QPainterPath *painterPath, qreal dx,
                             qreal dy, QPainterPath &out) {
  out = painterPath->translated(dx, dy);
}
void QPainterPath_united(const QPainterPath *painterPath, const QPainterPath &p,
                         QPainterPath &out) {
  out = painterPath->united(p);
}

//=============================================================================
void QPdfWriter_destructor(QPdfWriter *pdfWriter) { pdfWriter->~QPdfWriter(); }

//=============================================================================
void QPen_destructor(QPen *pen) { pen->~QPen(); }
void QPen_constructor(QPen *pen) { new (pen) QPen; }
void QPen_setWidth(QPen *pen, qreal width) { pen->setWidthF(width); }
void QPen_setBrush(QPen *pen, const QBrush& brush) { pen->setBrush(brush); }

//=============================================================================
void QPolygon_destructor(QPolygon *polygon) { polygon->~QPolygon(); }
void QPolygonF_destructor(QPolygonF *polygon) { polygon->~QPolygonF(); }
void QRegion_destructor(QRegion *region) { region->~QRegion(); }
void QTransform_destructor(QTransform *transform) { transform->~QTransform(); }

//=============================================================================
void QFont_constructor(QFont *font) { new (font) QFont(); }
void QFont_destructor(QFont *font) { font->~QFont(); }
void QFont_swap(QFont *font, QFont &other) { font->swap(other); }
void QFont_setFamily(QFont *font, const QString &family) {
  font->setFamily(family);
}
void QFont_setFamilies(QFont *font, const QStringList &families) {
  font->setFamilies(families);
}
void QFont_setStyleName(QFont *font, const QString &styleName) {
  font->setStyleName(styleName);
}
void QFont_setPointSize(QFont *font, int pointSize) {
  font->setPointSize(pointSize);
}
void QFont_setPointSizeF(QFont *font, qreal pointSize) {
  font->setPointSizeF(pointSize);
}
void QFont_setPixelSize(QFont *font, int pixelSize) {
  font->setPixelSize(pixelSize);
}
void QFont_setWeight(QFont *font, int weight) { font->setWeight(weight); }
void QFont_setBold(QFont *font, bool bold) { font->setBold(bold); }
void QFont_setStyle(QFont *font, QFont::Style style) { font->setStyle(style); }
void QFont_setItalic(QFont *font, bool italic) { font->setItalic(italic); }
void QFont_setUnderline(QFont *font, bool underline) {
  font->setUnderline(underline);
}
void QFont_setOverline(QFont *font, bool overline) {
  font->setOverline(overline);
}
void QFont_setStrikeOut(QFont *font, bool strikeout) {
  font->setStrikeOut(strikeout);
}
void QFont_setFixedPitch(QFont *font, bool fixedPitch) {
  font->setFixedPitch(fixedPitch);
}
void QFont_setKerning(QFont *font, bool kerning) { font->setKerning(kerning); }
void QFont_setStyleHint(QFont *font, QFont::StyleHint styleHint,
                        QFont::StyleStrategy styleStrategy) {
  font->setStyleHint(styleHint, styleStrategy);
}
void QFont_setStyleStrategy(QFont *font, QFont::StyleStrategy s) {
  font->setStyleStrategy(s);
}
void QFont_setStretch(QFont *font, int stretch) { font->setStretch(stretch); }
void QFont_setLetterSpacing(QFont *font, QFont::SpacingType type,
                            qreal spacing) {
  font->setLetterSpacing(type, spacing);
}
void QFont_setWordSpacing(QFont *font, qreal spacing) {
  font->setWordSpacing(spacing);
}
void QFont_setCapitalization(QFont *font,
                             QFont::Capitalization capitalization) {
  font->setCapitalization(capitalization);
}
void QFont_setHintingPreference(QFont *font,
                                QFont::HintingPreference hintingPreference) {
  font->setHintingPreference(hintingPreference);
}
bool QFont_exactMatch(const QFont *font) { return font->exactMatch(); }
bool QFont_isCopyOf(const QFont *font, const QFont &other) {
  return font->isCopyOf(other);
}
void QFont_key(const QFont *font, QString &out) { out = font->key(); }

//=============================================================================
void QFontInfo_destructor(QFontInfo *fontInfo) { fontInfo->~QFontInfo(); }
void QFontMetrics_destructor(QFontMetrics *fontMetrics) {
  fontMetrics->~QFontMetrics();
}
void QFontMetricsF_destructor(QFontMetricsF *fontMetrics) {
  fontMetrics->~QFontMetricsF();
}
void QGlyphRun_destructor(QGlyphRun *glyphRun) { glyphRun->~QGlyphRun(); }
void QStaticText_destructor(QStaticText *staticText) {
  staticText->~QStaticText();
}
void QTextDocument_destructor(QTextDocument *textDocument) {
  textDocument->~QTextDocument();
}
void QGraphicsItem_destructor(QGraphicsItem *graphicsItem) {
  graphicsItem->~QGraphicsItem();
}
void QGraphicsItem_delete(QGraphicsItem *graphicsItem) { delete graphicsItem; }
QGraphicsLineItem *QGraphicsLineItem_new() { return new QGraphicsLineItem(); }
void QGraphicsLineItem_destructor(QGraphicsLineItem *graphicsLineItem) {
  graphicsLineItem->~QGraphicsLineItem();
}
void QGraphicsLineItem_delete(QGraphicsLineItem *graphicsLineItem) {
  delete graphicsLineItem;
}
void QGraphicsObject_destructor(QGraphicsObject *graphicsObject) {
  graphicsObject->~QGraphicsObject();
}
void QGraphicsObject_delete(QGraphicsObject *graphicsObject) {
  delete graphicsObject;
}
QGraphicsView *QGraphicsView_new() { return new QGraphicsView(); }
void QGraphicsView_destructor(QGraphicsView *graphicsView) {
  graphicsView->~QGraphicsView();
}
void QGraphicsView_delete(QGraphicsView *graphicsView) { delete graphicsView; }
QGraphicsWidget *QGraphicsWidget_new() { return new QGraphicsWidget(); }
void QGraphicsWidget_destructor(QGraphicsWidget *graphicsWidget) {
  graphicsWidget->~QGraphicsWidget();
}
void QGraphicsWidget_delete(QGraphicsWidget *graphicsWidget) {
  delete graphicsWidget;
}
QListView *QListView_new() { return new QListView(); }
void QListView_destructor(QListView *listView) { listView->~QListView(); }
void QListView_delete(QListView *listView) { delete listView; }
QTableView *QTableView_new() { return new QTableView(); }
void QTableView_destructor(QTableView *tableView) { tableView->~QTableView(); }
void QTableView_delete(QTableView *tableView) { delete tableView; }
QTreeView *QTreeView_new() { return new QTreeView(); }
void QTreeView_destructor(QTreeView *treeView) { treeView->~QTreeView(); }
void QTreeView_delete(QTreeView *treeView) { delete treeView; }

//=============================================================================
void QBoxLayout_destructor(QBoxLayout *boxLayout) { boxLayout->~QBoxLayout(); }
void QBoxLayout_delete(QBoxLayout *boxLayout) { delete boxLayout; }
void QBoxLayout_addStretch(QBoxLayout *boxLayout, int stretch) {
  boxLayout->addStretch(stretch);
}
void QBoxLayout_addSpacerItem(QBoxLayout *boxLayout, QSpacerItem *spacerItem) {
  boxLayout->addSpacerItem(spacerItem);
}
void QBoxLayout_addWidget(QBoxLayout *boxLayout, QWidget *widget, int stretch,
                          Qt::Alignment alignment) {
  boxLayout->addWidget(widget, stretch, alignment);
}
void QBoxLayout_addLayout(QBoxLayout *boxLayout, QLayout *layout, int stretch) {
  boxLayout->addLayout(layout, stretch);
}
void QBoxLayout_addStrut(QBoxLayout *boxLayout, int size) {
  boxLayout->addStrut(size);
}
void QBoxLayout_addItem(QBoxLayout *boxLayout, QLayoutItem *item) {
  boxLayout->addItem(item);
}
void QBoxLayout_insertSpacing(QBoxLayout *boxLayout, int index, int size) {
  boxLayout->insertSpacing(index, size);
}
void QBoxLayout_insertStretch(QBoxLayout *boxLayout, int index, int stretch) {
  boxLayout->insertStretch(index, stretch);
}
void QBoxLayout_insertSpacerItem(QBoxLayout *boxLayout, int index,
                                 QSpacerItem *spacerItem) {
  boxLayout->insertSpacerItem(index, spacerItem);
}
void QBoxLayout_insertWidget(QBoxLayout *boxLayout, int index, QWidget *widget,
                             int stretch, Qt::Alignment alignment) {
  boxLayout->insertWidget(index, widget, stretch, alignment);
}
void QBoxLayout_insertLayout(QBoxLayout *boxLayout, int index, QLayout *layout,
                             int stretch) {
  boxLayout->insertLayout(index, layout, stretch);
}
void QBoxLayout_insertItem(QBoxLayout *boxLayout, int index,
                           QLayoutItem *item) {
  boxLayout->insertItem(index, item);
}
void QBoxLayout_setSpacing(QBoxLayout *boxLayout, int spacing) {
  boxLayout->setSpacing(spacing);
}
bool QBoxLayout_setStretchFactor(QBoxLayout *boxLayout, QWidget *w,
                                 int stretch) {
  return boxLayout->setStretchFactor(w, stretch);
}
bool QBoxLayout_setStretchFactor1(QBoxLayout *boxLayout, QLayout *l,
                                  int stretch) {
  return boxLayout->setStretchFactor(l, stretch);
}
void QBoxLayout_setStretch(QBoxLayout *boxLayout, int index, int stretch) {
  boxLayout->setStretch(index, stretch);
}

//=============================================================================
QHBoxLayout *QHBoxLayout_new() { return new QHBoxLayout(); }
void QHBoxLayout_destructor(QHBoxLayout *hboxLayout) {
  hboxLayout->~QHBoxLayout();
}
void QHBoxLayout_delete(QHBoxLayout *hboxLayout) { delete hboxLayout; }
QVBoxLayout *QVBoxLayout_new() { return new QVBoxLayout(); }
void QVBoxLayout_destructor(QVBoxLayout *vboxLayout) {
  vboxLayout->~QVBoxLayout();
}
void QVBoxLayout_delete(QVBoxLayout *vboxLayout) { delete vboxLayout; }

//=============================================================================
QFormLayout *QFormLayout_new() { return new QFormLayout(); }
void QFormLayout_destructor(QFormLayout *formLayout) {
  formLayout->~QFormLayout();
}
void QFormLayout_delete(QFormLayout *formLayout) { delete formLayout; }
void QFormLayout_setFieldGrowthPolicy(QFormLayout *formLayout,
                                      QFormLayout::FieldGrowthPolicy policy) {
  formLayout->setFieldGrowthPolicy(policy);
}
void QFormLayout_setRowWrapPolicy(QFormLayout *formLayout,
                                  QFormLayout::RowWrapPolicy policy) {
  formLayout->setRowWrapPolicy(policy);
}
void QFormLayout_setLabelAlignment(QFormLayout *formLayout,
                                   Qt::Alignment alignment) {
  formLayout->setLabelAlignment(alignment);
}
void QFormLayout_setFormAlignment(QFormLayout *formLayout,
                                  Qt::Alignment alignment) {
  formLayout->setFormAlignment(alignment);
}
void QFormLayout_setHorizontalSpacing(QFormLayout *formLayout, int spacing) {
  formLayout->setHorizontalSpacing(spacing);
}
void QFormLayout_setVerticalSpacing(QFormLayout *formLayout, int spacing) {
  formLayout->setVerticalSpacing(spacing);
}
void QFormLayout_setSpacing(QFormLayout *formLayout, int spacing) {
  formLayout->setSpacing(spacing);
}
void QFormLayout_addRow(QFormLayout *formLayout, QWidget *label,
                        QWidget *field) {
  formLayout->addRow(label, field);
}
void QFormLayout_addRow1(QFormLayout *formLayout, QWidget *label,
                         QLayout *field) {
  formLayout->addRow(label, field);
}
void QFormLayout_addRow2(QFormLayout *formLayout, const QString &labelText,
                         QWidget *field) {
  formLayout->addRow(labelText, field);
}
void QFormLayout_addRow3(QFormLayout *formLayout, const QString &labelText,
                         QLayout *field) {
  formLayout->addRow(labelText, field);
}
void QFormLayout_addRow4(QFormLayout *formLayout, QWidget *widget) {
  formLayout->addRow(widget);
}
void QFormLayout_addRow5(QFormLayout *formLayout, QLayout *layout) {
  formLayout->addRow(layout);
}
void QFormLayout_insertRow(QFormLayout *formLayout, int row, QWidget *label,
                           QWidget *field) {
  formLayout->insertRow(row, label, field);
}
void QFormLayout_insertRow1(QFormLayout *formLayout, int row, QWidget *label,
                            QLayout *field) {
  formLayout->insertRow(row, label, field);
}
void QFormLayout_insertRow2(QFormLayout *formLayout, int row,
                            const QString &labelText, QWidget *field) {
  formLayout->insertRow(row, labelText, field);
}
void QFormLayout_insertRow3(QFormLayout *formLayout, int row,
                            const QString &labelText, QLayout *field) {
  formLayout->insertRow(row, labelText, field);
}
void QFormLayout_insertRow4(QFormLayout *formLayout, int row, QWidget *widget) {
  formLayout->insertRow(row, widget);
}
void QFormLayout_insertRow5(QFormLayout *formLayout, int row, QLayout *layout) {
  formLayout->insertRow(row, layout);
}
void QFormLayout_removeRow(QFormLayout *formLayout, int row) {
  formLayout->removeRow(row);
}
void QFormLayout_removeRow1(QFormLayout *formLayout, QWidget *widget) {
  formLayout->removeRow(widget);
}
void QFormLayout_removeRow2(QFormLayout *formLayout, QLayout *layout) {
  formLayout->removeRow(layout);
}
void QFormLayout_setItem(QFormLayout *formLayout, int row,
                         QFormLayout::ItemRole role, QLayoutItem *item) {
  formLayout->setItem(row, role, item);
}
void QFormLayout_setWidget(QFormLayout *formLayout, int row,
                           QFormLayout::ItemRole role, QWidget *widget) {
  formLayout->setWidget(row, role, widget);
}
void QFormLayout_setLayout(QFormLayout *formLayout, int row,
                           QFormLayout::ItemRole role, QLayout *layout) {
  formLayout->setLayout(row, role, layout);
}

//=============================================================================
void QLayout_destructor(QLayout *layout) { layout->~QLayout(); }
void QLayout_delete(QLayout *layout) { delete layout; }
void QLayout_setContentsMargins(QLayout *layout, int left, int top, int right, int bottom) {
    layout->setContentsMargins(left, top, right, bottom);
}

//=============================================================================
QWidget *QWidget_new() { return new QWidget(); }
void QWidget_destructor(QWidget *widget) { widget->~QWidget(); }
void QWidget_delete(QWidget *widget) { delete widget; }
void QWidget_setStyle(QWidget *widget, QStyle *style) {
  widget->setStyle(style);
}
void QWidget_setEnabled(QWidget *widget, bool enabled) {
  widget->setEnabled(enabled);
}
void QWidget_setDisabled(QWidget *widget, bool disabled) {
  widget->setDisabled(disabled);
}
void QWidget_setMinimumSize(QWidget *widget, int minw, int minh) {
  widget->setMinimumSize(minw, minh);
}
void QWidget_setMaximumSize(QWidget *widget, int maxw, int maxh) {
  widget->setMaximumSize(maxw, maxh);
}
void QWidget_setMinimumWidth(QWidget *widget, int minw) {
  widget->setMinimumWidth(minw);
}
void QWidget_setMinimumHeight(QWidget *widget, int minh) {
  widget->setMinimumHeight(minh);
}
void QWidget_setMaximumWidth(QWidget *widget, int maxw) {
  widget->setMaximumWidth(maxw);
}
void QWidget_setMaximumHeight(QWidget *widget, int maxh) {
  widget->setMaximumHeight(maxh);
}
void QWidget_setFixedSize(QWidget *widget, int w, int h) {
  widget->setFixedSize(w, h);
}
void QWidget_setFixedWidth(QWidget *widget, int w) { widget->setFixedWidth(w); }
void QWidget_setFixedHeight(QWidget *widget, int h) {
  widget->setFixedHeight(h);
}
void QWidget_mapToGlobal(const QWidget *widget, const QPoint &p, QPoint &out) {
  out = widget->mapToGlobal(p);
}
void QWidget_mapFromGlobal(const QWidget *widget, const QPoint &p,
                           QPoint &out) {
  out = widget->mapFromGlobal(p);
}
void QWidget_mapToParent(const QWidget *widget, const QPoint &p, QPoint &out) {
  out = widget->mapToParent(p);
}
void QWidget_mapFromParent(const QWidget *widget, const QPoint &p,
                           QPoint &out) {
  out = widget->mapFromParent(p);
}
void QWidget_mapTo(const QWidget *widget, const QWidget *other, const QPoint &p,
                   QPoint &out) {
  out = widget->mapTo(other, p);
}
void QWidget_mapFrom(const QWidget *widget, const QWidget *other,
                     const QPoint &p, QPoint &out) {
  out = widget->mapFrom(other, p);
}
void QWidget_setFont(QWidget *widget, const QFont &font) {
  widget->setFont(font);
}
void QWidget_setCursor(QWidget *widget, const QCursor &cursor) {
  widget->setCursor(cursor);
}
void QWidget_unsetCursor(QWidget *widget) { widget->unsetCursor(); }
void QWidget_setMask(QWidget *widget, const QRegion &region) {
  widget->setMask(region);
}
void QWidget_clearMask(QWidget *widget) { widget->clearMask(); }
void QWidget_setWindowIcon(QWidget *widget, const QIcon &icon) {
  widget->setWindowIcon(icon);
}
void QWidget_setWindowIconText(QWidget *widget, const QString &windowIconText) {
  widget->setWindowIconText(windowIconText);
}
void QWidget_setToolTip(QWidget *widget, const QString &toolTip) {
  widget->setToolTip(toolTip);
}
void QWidget_setToolTipDuration(QWidget *widget, int msec) {
  widget->setToolTipDuration(msec);
}
void QWidget_setStatusTip(QWidget *widget, const QString &statusTip) {
  widget->setStatusTip(statusTip);
}
void QWidget_setWhatsThis(QWidget *widget, const QString &whatsThis) {
  widget->setWhatsThis(whatsThis);
}
void QWidget_setAccessibleName(QWidget *widget, const QString &name) {
  widget->setAccessibleName(name);
}
void QWidget_setAccessibleDescription(QWidget *widget,
                                      const QString &description) {
  widget->setAccessibleDescription(description);
}
void QWidget_setLayoutDirection(QWidget *widget,
                                Qt::LayoutDirection direction) {
  widget->setLayoutDirection(direction);
}
void QWidget_unsetLayoutDirection(QWidget *widget) {
  widget->unsetLayoutDirection();
}
void QWidget_setLocale(QWidget *widget, const QLocale &locale) {
  widget->setLocale(locale);
}
void QWidget_unsetLocale(QWidget *widget) { widget->unsetLocale(); }
bool QWidget_isActiveWindow(const QWidget *widget) {
  return widget->isActiveWindow();
}
void QWidget_activateWindow(QWidget *widget) { widget->activateWindow(); }
void QWidget_clearFocus(QWidget *widget) { widget->clearFocus(); }
void QWidget_setFocus(QWidget *widget, Qt::FocusReason reason) {
  widget->setFocus(reason);
}
void QWidget_setFocusPolicy(QWidget *widget, Qt::FocusPolicy policy) {
  widget->setFocusPolicy(policy);
}
bool QWidget_hasFocus(const QWidget *widget) { return widget->hasFocus(); }
void QWidget_setContextMenuPolicy(QWidget *widget,
                                  Qt::ContextMenuPolicy policy) {
  widget->setContextMenuPolicy(policy);
}
void QWidget_grabMouse(QWidget *widget) { widget->grabMouse(); }
void QWidget_grabMouse(QWidget *widget, const QCursor &cursor) {
  widget->grabMouse(cursor);
}
void QWidget_releaseMouse(QWidget *widget) { widget->releaseMouse(); }
void QWidget_grabKeyboard(QWidget *widget) { widget->grabKeyboard(); }
void QWidget_releaseKeyboard(QWidget *widget) { widget->releaseKeyboard(); }
int QWidget_grabShortcut(QWidget *widget, const QKeySequence &key,
                         Qt::ShortcutContext context) {
  return widget->grabShortcut(key, context);
}
void QWidget_releaseShortcut(QWidget *widget, int id) {
  widget->releaseShortcut(id);
}
void QWidget_setShortcutEnabled(QWidget *widget, int id, bool enable) {
  widget->setShortcutEnabled(id, enable);
}
void QWidget_setShortcutAutoRepeat(QWidget *widget, int id, bool enable) {
  widget->setShortcutAutoRepeat(id, enable);
}
void QWidget_update(QWidget *widget) { widget->update(); }
void QWidget_repaint(QWidget *widget) { widget->repaint(); }
bool QWidget_isVisible(const QWidget* widget) {
	return widget->isVisible();
}
void QWidget_setVisible(QWidget *widget, bool visible) {
  widget->setVisible(visible);
}
void QWidget_setHidden(QWidget *widget, bool hidden) {
  widget->setHidden(hidden);
}
void QWidget_show(QWidget *widget) { widget->show(); }
void QWidget_hide(QWidget *widget) { widget->hide(); }
void QWidget_showMinimized(QWidget *widget) { widget->showMinimized(); }
void QWidget_showMaximized(QWidget *widget) { widget->showMaximized(); }
void QWidget_showFullScreen(QWidget *widget) { widget->showFullScreen(); }
void QWidget_showNormal(QWidget *widget) { widget->showNormal(); }
bool QWidget_close(QWidget *widget) { return widget->close(); }
void QWidget_raise(QWidget *widget) { widget->raise(); }
void QWidget_lower(QWidget *widget) { widget->lower(); }
void QWidget_stackUnder(QWidget *widget, QWidget *other) {
  widget->stackUnder(other);
}
void QWidget_move(QWidget *widget, int x, int y) { widget->move(x, y); }
void QWidget_resize(QWidget *widget, int w, int h) { widget->resize(w, h); }
void QWidget_setGeometry(QWidget *widget, int x, int y, int w, int h) {
  widget->setGeometry(x, y, w, h);
}
void QWidget_setSizePolicy(QWidget *widget, QSizePolicy::Policy horizontal,
                           QSizePolicy::Policy vertical) {
  widget->setSizePolicy(horizontal, vertical);
}
void QWidget_setContentsMargins(QWidget *widget, int left, int top, int right,
                                int bottom) {
  widget->setContentsMargins(left, top, right, bottom);
}
void QWidget_setLayout(QWidget *widget, QLayout *layout) {
  widget->setLayout(layout);
}
void QWidget_setParent(QWidget *widget, QWidget *parent) {
  widget->setParent(parent);
}
QObject *QWidget_upcast_QObject(QWidget *self) {
  return static_cast<QObject *>(self);
}
QPaintDevice *QWidget_upcast_QPaintDevice(QWidget *self) {
  return static_cast<QPaintDevice *>(self);
}
//=============================================================================
void QAbstractButton_destructor(QAbstractButton *abstractButton) {
  abstractButton->~QAbstractButton();
}
void QAbstractButton_delete(QAbstractButton *abstractButton) {
  delete abstractButton;
}
void QAbstractButton_setText(QAbstractButton *abstractButton,
                             const QString &text) {
  abstractButton->setText(text);
}
void QAbstractButton_setIcon(QAbstractButton *abstractButton,
                             const QIcon &icon) {
  abstractButton->setIcon(icon);
}
void QAbstractButton_setShortcut(QAbstractButton *abstractButton,
                                 const QKeySequence &key) {
  abstractButton->setShortcut(key);
}
void QAbstractButton_setCheckable(QAbstractButton *abstractButton,
                                  bool checkable) {
  abstractButton->setCheckable(checkable);
}
bool QAbstractButton_isChecked(const QAbstractButton *abstractButton) {
  return abstractButton->isChecked();
}
void QAbstractButton_setDown(QAbstractButton *abstractButton, bool down) {
  abstractButton->setDown(down);
}
bool QAbstractButton_isDown(const QAbstractButton *abstractButton) {
  return abstractButton->isDown();
}
void QAbstractButton_setAutoRepeat(QAbstractButton *abstractButton,
                                   bool autoRepeat) {
  abstractButton->setAutoRepeat(autoRepeat);
}
void QAbstractButton_setAutoRepeatDelay(QAbstractButton *abstractButton,
                                        int autoRepeatDelay) {
  abstractButton->setAutoRepeatDelay(autoRepeatDelay);
}
void QAbstractButton_setAutoRepeatInterval(QAbstractButton *abstractButton,
                                           int autoRepeatInterval) {
  abstractButton->setAutoRepeatInterval(autoRepeatInterval);
}
void QAbstractButton_setAutoExclusive(QAbstractButton *abstractButton,
                                      bool autoExclusive) {
  abstractButton->setAutoExclusive(autoExclusive);
}
void QAbstractButton_setIconSize(QAbstractButton *abstractButton,
                                 const QSize &size) {
  abstractButton->setIconSize(size);
}
void QAbstractButton_animateClick(QAbstractButton *abstractButton, int msec) {
  abstractButton->animateClick(msec);
}
void QAbstractButton_click(QAbstractButton *abstractButton) {
  abstractButton->click();
}
void QAbstractButton_toggle(QAbstractButton *abstractButton) {
  abstractButton->toggle();
}
void QAbstractButton_setChecked(QAbstractButton *abstractButton, bool checked) {
  abstractButton->setChecked(checked);
}

//=============================================================================
QAbstractScrollArea *QAbstractScrollArea_new() {
  return new QAbstractScrollArea();
}
void QAbstractScrollArea_destructor(QAbstractScrollArea *abstractScrollArea) {
  abstractScrollArea->~QAbstractScrollArea();
}
void QAbstractScrollArea_delete(QAbstractScrollArea *abstractScrollArea) {
  delete abstractScrollArea;
}
QAbstractSlider *QAbstractSlider_new() { return new QAbstractSlider(); }
void QAbstractSlider_destructor(QAbstractSlider *abstractSlider) {
  abstractSlider->~QAbstractSlider();
}
void QAbstractSlider_delete(QAbstractSlider *abstractSlider) {
  delete abstractSlider;
}
QButtonGroup *QButtonGroup_new() { return new QButtonGroup(); }
void QButtonGroup_destructor(QButtonGroup *buttonGroup) {
  buttonGroup->~QButtonGroup();
}
void QButtonGroup_delete(QButtonGroup *buttonGroup) { delete buttonGroup; }

//=============================================================================
QCheckBox *QCheckBox_new() { return new QCheckBox(); }
Qt::CheckState QCheckBox_checkState(const QCheckBox* checkBox) { return checkBox->checkState(); }
void QCheckBox_setCheckState(QCheckBox* checkBox, Qt::CheckState checkState) { checkBox->setCheckState(checkState); }
void QCheckBox_destructor(QCheckBox *checkBox) { checkBox->~QCheckBox(); }
void QCheckBox_delete(QCheckBox *checkBox) { delete checkBox; }

//=============================================================================
QComboBox *QComboBox_new() { return new QComboBox(); }
void QComboBox_destructor(QComboBox *comboBox) { comboBox->~QComboBox(); }
void QComboBox_delete(QComboBox *comboBox) { delete comboBox; }
void QComboBox_setMaxVisibleItems(QComboBox *comboBox, int maxItems) {
  comboBox->setMaxVisibleItems(maxItems);
}
int QComboBox_count(const QComboBox *comboBox) { return comboBox->count(); }
void QComboBox_setMaxCount(QComboBox *comboBox, int max) {
  comboBox->setMaxCount(max);
}
void QComboBox_setFrame(QComboBox *comboBox, bool frame) {
  comboBox->setFrame(frame);
}
void QComboBox_setInsertPolicy(QComboBox *comboBox,
                               QComboBox::InsertPolicy policy) {
  comboBox->setInsertPolicy(policy);
}
void QComboBox_setSizeAdjustPolicy(QComboBox *comboBox,
                                   QComboBox::SizeAdjustPolicy policy) {
  comboBox->setSizeAdjustPolicy(policy);
}
void QComboBox_setMinimumContentsLength(QComboBox *comboBox, int characters) {
  comboBox->setMinimumContentsLength(characters);
}
void QComboBox_setIconSize(QComboBox *comboBox, const QSize &size) {
  comboBox->setIconSize(size);
}
void QComboBox_setEditable(QComboBox *comboBox, bool editable) {
  comboBox->setEditable(editable);
}
int QComboBox_currentIndex(const QComboBox *comboBox) {
  return comboBox->currentIndex();
}
void QComboBox_addItem(QComboBox *comboBox, const QString &text,
                       const QVariant &userData) {
  comboBox->addItem(text, userData);
}
void QComboBox_addItem1(QComboBox *comboBox, const QIcon &icon,
                        const QString &text, const QVariant &userData) {
  comboBox->addItem(icon, text, userData);
}
void QComboBox_insertItem(QComboBox *comboBox, int index, const QString &text,
                          const QVariant &userData) {
  comboBox->insertItem(index, text, userData);
}
void QComboBox_insertItem1(QComboBox *comboBox, int index, const QIcon &icon,
                           const QString &text, const QVariant &userData) {
  comboBox->insertItem(index, icon, text, userData);
}
void QComboBox_insertSeparator(QComboBox *comboBox, int index) {
  comboBox->insertSeparator(index);
}
void QComboBox_removeItem(QComboBox *comboBox, int index) {
  comboBox->removeItem(index);
}
void QComboBox_setItemText(QComboBox *comboBox, int index,
                           const QString &text) {
  comboBox->setItemText(index, text);
}
void QComboBox_setItemIcon(QComboBox *comboBox, int index, const QIcon &icon) {
  comboBox->setItemIcon(index, icon);
}
void QComboBox_setItemData(QComboBox *comboBox, int index,
                           const QVariant &value, int role) {
  comboBox->setItemData(index, value, role);
}
void QComboBox_clear(QComboBox *comboBox) { comboBox->clear(); }
void QComboBox_clearEditText(QComboBox *comboBox) { comboBox->clearEditText(); }
void QComboBox_setEditText(QComboBox *comboBox, const QString &text) {
  comboBox->setEditText(text);
}
void QComboBox_setCurrentIndex(QComboBox *comboBox, int index) {
  comboBox->setCurrentIndex(index);
}
void QComboBox_setCurrentText(QComboBox *comboBox, const QString &text) {
  comboBox->setCurrentText(text);
}

QDateEdit *QDateEdit_new() { return new QDateEdit(); }
void QDateEdit_destructor(QDateEdit *dateEdit) { dateEdit->~QDateEdit(); }
void QDateEdit_delete(QDateEdit *dateEdit) { delete dateEdit; }
QDateTimeEdit *QDateTimeEdit_new() { return new QDateTimeEdit(); }
void QDateTimeEdit_destructor(QDateTimeEdit *dateTimeEdit) {
  dateTimeEdit->~QDateTimeEdit();
}
void QDateTimeEdit_delete(QDateTimeEdit *dateTimeEdit) { delete dateTimeEdit; }
QTimeEdit *QTimeEdit_new() { return new QTimeEdit(); }
void QTimeEdit_destructor(QTimeEdit *timeEdit) { timeEdit->~QTimeEdit(); }
void QTimeEdit_delete(QTimeEdit *timeEdit) { delete timeEdit; }
QDockWidget *QDockWidget_new() { return new QDockWidget(); }
void QDockWidget_destructor(QDockWidget *dockWidget) {
  dockWidget->~QDockWidget();
}
void QDockWidget_delete(QDockWidget *dockWidget) { delete dockWidget; }
QFontComboBox *QFontComboBox_new() { return new QFontComboBox(); }
void QFontComboBox_destructor(QFontComboBox *fontComboBox) {
  fontComboBox->~QFontComboBox();
}
void QFontComboBox_delete(QFontComboBox *fontComboBox) { delete fontComboBox; }
QFrame *QFrame_new() { return new QFrame(); }
void QFrame_destructor(QFrame *frame) { frame->~QFrame(); }
void QFrame_delete(QFrame *frame) { delete frame; }
QGroupBox *QGroupBox_new() { return new QGroupBox(); }
void QGroupBox_destructor(QGroupBox *groupBox) { groupBox->~QGroupBox(); }
void QGroupBox_delete(QGroupBox *groupBox) { delete groupBox; }

//=============================================================================
QLabel *QLabel_new() { return new QLabel(); }
void QLabel_destructor(QLabel *label) { label->~QLabel(); }
void QLabel_delete(QLabel *label) { delete label; }
void QLabel_text(const QLabel *label, QString &out) { out = label->text(); }
void QLabel_setTextFormat(QLabel *label, Qt::TextFormat textFormat) {
  label->setTextFormat(textFormat);
}
void QLabel_setAlignment(QLabel *label, Qt::Alignment alignment) {
  label->setAlignment(alignment);
}
void QLabel_setWordWrap(QLabel *label, bool on) { label->setWordWrap(on); }
void QLabel_setIndent(QLabel *label, int indent) { label->setIndent(indent); }
void QLabel_setMargin(QLabel *label, int margin) { label->setMargin(margin); }
void QLabel_setScaledContents(QLabel *label, bool scaledContents) {
  label->setScaledContents(scaledContents);
}
void QLabel_setBuddy(QLabel *label, QWidget *buddy) { label->setBuddy(buddy); }
void QLabel_setOpenExternalLinks(QLabel *label, bool open) {
  label->setOpenExternalLinks(open);
}
void QLabel_setTextInteractionFlags(QLabel *label,
                                    Qt::TextInteractionFlags flags) {
  label->setTextInteractionFlags(flags);
}
void QLabel_setSelection(QLabel *label, int start, int length) {
  label->setSelection(start, length);
}
bool QLabel_hasSelectedText(const QLabel *label) {
  return label->hasSelectedText();
}
void QLabel_selectedText(const QLabel *label, QString &out) {
  out = label->selectedText();
}
int QLabel_selectionStart(const QLabel *label) {
  return label->selectionStart();
}
void QLabel_setText(QLabel *label, const QString &text) {
  label->setText(text);
}
void QLabel_setPixmap(QLabel *label, const QPixmap &pixmap) {
  label->setPixmap(pixmap);
}
void QLabel_clear(QLabel *label) { label->clear(); }

//=============================================================================
QLineEdit *QLineEdit_new() { return new QLineEdit(); }
void QLineEdit_destructor(QLineEdit *lineEdit) { lineEdit->~QLineEdit(); }
void QLineEdit_delete(QLineEdit *lineEdit) { delete lineEdit; }
void QLineEdit_text(const QLineEdit *lineEdit, QString &out) {
  out = lineEdit->text();
}
void QLineEdit_displayText(const QLineEdit *lineEdit, QString &out) {
  out = lineEdit->displayText();
}
void QLineEdit_setPlaceholderText(QLineEdit *lineEdit,
                                  const QString &placeholder) {
  lineEdit->setPlaceholderText(placeholder);
}
void QLineEdit_setMaxLength(QLineEdit *lineEdit, int maxLength) {
  lineEdit->setMaxLength(maxLength);
}
void QLineEdit_setFrame(QLineEdit *lineEdit, bool frame) {
  lineEdit->setFrame(frame);
}
void QLineEdit_setClearButtonEnabled(QLineEdit *lineEdit, bool enable) {
  lineEdit->setClearButtonEnabled(enable);
}
void QLineEdit_setEchoMode(QLineEdit *lineEdit, QLineEdit::EchoMode echoMode) {
  lineEdit->setEchoMode(echoMode);
}
void QLineEdit_setReadOnly(QLineEdit *lineEdit, bool readonly) {
  lineEdit->setReadOnly(readonly);
}
int QLineEdit_cursorPosition(const QLineEdit *lineEdit) {
  return lineEdit->cursorPosition();
}
void QLineEdit_setCursorPosition(QLineEdit *lineEdit, int pos) {
  lineEdit->setCursorPosition(pos);
}
int QLineEdit_cursorPositionAt(QLineEdit *lineEdit, const QPoint &pos) {
  return lineEdit->cursorPositionAt(pos);
}
void QLineEdit_setAlignment(QLineEdit *lineEdit, Qt::Alignment alignment) {
  lineEdit->setAlignment(alignment);
}
void QLineEdit_cursorForward(QLineEdit *lineEdit, bool mark, int steps) {
  lineEdit->cursorForward(mark, steps);
}
void QLineEdit_cursorBackward(QLineEdit *lineEdit, bool mark, int steps) {
  lineEdit->cursorBackward(mark, steps);
}
void QLineEdit_cursorWordForward(QLineEdit *lineEdit, bool mark) {
  lineEdit->cursorWordForward(mark);
}
void QLineEdit_cursorWordBackward(QLineEdit *lineEdit, bool mark) {
  lineEdit->cursorWordBackward(mark);
}
void QLineEdit_backspace(QLineEdit *lineEdit) { lineEdit->backspace(); }
void QLineEdit_del(QLineEdit *lineEdit) { lineEdit->del(); }
void QLineEdit_home(QLineEdit *lineEdit, bool mark) { lineEdit->home(mark); }
void QLineEdit_end(QLineEdit *lineEdit, bool mark) { lineEdit->end(mark); }
bool QLineEdit_isModified(const QLineEdit *lineEdit) {
  return lineEdit->isModified();
}
void QLineEdit_setModified(QLineEdit *lineEdit, bool modified) {
  lineEdit->setModified(modified);
}
void QLineEdit_setSelection(QLineEdit *lineEdit, int start, int length) {
  lineEdit->setSelection(start, length);
}
bool QLineEdit_hasSelectedText(QLineEdit *lineEdit) {
  return lineEdit->hasSelectedText();
}
void QLineEdit_selectedText(const QLineEdit *lineEdit, QString &out) {
  out = lineEdit->selectedText();
}
int QLineEdit_selectionStart(const QLineEdit *lineEdit) {
  return lineEdit->selectionStart();
}
int QLineEdit_selectionEnd(const QLineEdit *lineEdit) {
  return lineEdit->selectionEnd();
}
int QLineEdit_selectionLength(const QLineEdit *lineEdit) {
  return lineEdit->selectionLength();
}
bool QLineEdit_isUndoAvailable(const QLineEdit *lineEdit) {
  return lineEdit->isUndoAvailable();
}
bool QLineEdit_isRedoAvailable(const QLineEdit *lineEdit) {
  return lineEdit->isRedoAvailable();
}
void QLineEdit_setDragEnabled(QLineEdit *lineEdit, bool dragEnabled) {
  lineEdit->setDragEnabled(dragEnabled);
}
void QLineEdit_setCursorMoveStyle(QLineEdit *lineEdit,
                                  Qt::CursorMoveStyle style) {
  lineEdit->setCursorMoveStyle(style);
}
void QLineEdit_setInputMask(QLineEdit *lineEdit, const QString &inputMask) {
  lineEdit->setInputMask(inputMask);
}
bool QLineEdit_hasAcceptableInput(const QLineEdit *lineEdit) {
  return lineEdit->hasAcceptableInput();
}
void QLineEdit_setTextMargins(QLineEdit *lineEdit, int left, int top, int right,
                              int bottom) {
  lineEdit->setTextMargins(left, top, right, bottom);
}
void QLineEdit_setText(QLineEdit *lineEdit, const QString &text) {
  lineEdit->setText(text);
}
void QLineEdit_clear(QLineEdit *lineEdit) { lineEdit->clear(); }
void QLineEdit_selectAll(QLineEdit *lineEdit) { lineEdit->selectAll(); }
void QLineEdit_undo(QLineEdit *lineEdit) { lineEdit->undo(); }
void QLineEdit_redo(QLineEdit *lineEdit) { lineEdit->redo(); }
void QLineEdit_cut(QLineEdit *lineEdit) { lineEdit->cut(); }
void QLineEdit_copy(const QLineEdit *lineEdit) { lineEdit->copy(); }
void QLineEdit_paste(QLineEdit *lineEdit) { lineEdit->paste(); }
void QLineEdit_deselect(QLineEdit *lineEdit) { lineEdit->deselect(); }
void QLineEdit_insert(QLineEdit *lineEdit, const QString &text) {
  lineEdit->insert(text);
}
QMenu *QLineEdit_createStandardContextMenu(QLineEdit *lineEdit) {
  return lineEdit->createStandardContextMenu();
}

//=============================================================================

QMenu *QMenu_new() { return new QMenu(); }
void QMenu_destructor(QMenu *menu) { menu->~QMenu(); }
void QMenu_delete(QMenu *menu) { delete menu; }
QMenuBar *QMenuBar_new() { return new QMenuBar(); }
void QMenuBar_destructor(QMenuBar *menuBar) { menuBar->~QMenuBar(); }
void QMenuBar_delete(QMenuBar *menuBar) { delete menuBar; }
QPlainTextEdit *QPlainTextEdit_new() { return new QPlainTextEdit(); }
void QPlainTextEdit_destructor(QPlainTextEdit *plainTextEdit) {
  plainTextEdit->~QPlainTextEdit();
}
void QPlainTextEdit_delete(QPlainTextEdit *plainTextEdit) {
  delete plainTextEdit;
}
QProgressBar *QProgressBar_new() { return new QProgressBar(); }
void QProgressBar_destructor(QProgressBar *progressBar) {
  progressBar->~QProgressBar();
}
void QProgressBar_delete(QProgressBar *progressBar) { delete progressBar; }

//=============================================================================
QPushButton *QPushButton_new() { return new QPushButton(); }
void QPushButton_destructor(QPushButton *pushButton) {
  pushButton->~QPushButton();
}
void QPushButton_delete(QPushButton *pushButton) { delete pushButton; }
void QPushButton_setAutoDefault(QPushButton *pushButton, bool autoDefault) {
  pushButton->setAutoDefault(autoDefault);
}
void QPushButton_setDefault(QPushButton *pushButton, bool default_) {
  pushButton->setDefault(default_);
}
void QPushButton_setMenu(QPushButton *pushButton, QMenu *menu) {
  pushButton->setMenu(menu);
}
void QPushButton_setFlat(QPushButton *pushButton, bool flat) {
  pushButton->setFlat(flat);
}
void QPushButton_showMenu(QPushButton *pushButton) { pushButton->showMenu(); }

//=============================================================================
QRadioButton *QRadioButton_new() { return new QRadioButton(); }
void QRadioButton_destructor(QRadioButton *radioButton) {
  radioButton->~QRadioButton();
}
void QRadioButton_delete(QRadioButton *radioButton) { delete radioButton; }
QScrollArea *QScrollArea_new() { return new QScrollArea(); }
void QScrollArea_destructor(QScrollArea *scrollArea) {
  scrollArea->~QScrollArea();
}
void QScrollArea_delete(QScrollArea *scrollArea) { delete scrollArea; }
QScrollBar *QScrollBar_new() { return new QScrollBar(); }
void QScrollBar_destructor(QScrollBar *scrollBar) { scrollBar->~QScrollBar(); }
void QScrollBar_delete(QScrollBar *scrollBar) { delete scrollBar; }
QSlider *QSlider_new() { return new QSlider(); }
void QSlider_destructor(QSlider *slider) { slider->~QSlider(); }
void QSlider_delete(QSlider *slider) { delete slider; }
QDoubleSpinBox *QDoubleSpinBox_new() { return new QDoubleSpinBox(); }
void QDoubleSpinBox_destructor(QDoubleSpinBox *doubleSpinBox) {
  doubleSpinBox->~QDoubleSpinBox();
}
void QDoubleSpinBox_delete(QDoubleSpinBox *doubleSpinBox) {
  delete doubleSpinBox;
}
QSpinBox *QSpinBox_new() { return new QSpinBox(); }
void QSpinBox_destructor(QSpinBox *spinBox) { spinBox->~QSpinBox(); }
void QSpinBox_delete(QSpinBox *spinBox) { delete spinBox; }
QStatusBar *QStatusBar_new() { return new QStatusBar(); }
void QStatusBar_destructor(QStatusBar *statusBar) { statusBar->~QStatusBar(); }
void QStatusBar_delete(QStatusBar *statusBar) { delete statusBar; }
QTextEdit *QTextEdit_new() { return new QTextEdit(); }
void QTextEdit_destructor(QTextEdit *textEdit) { textEdit->~QTextEdit(); }
void QTextEdit_delete(QTextEdit *textEdit) { delete textEdit; }

//

MQPaintEventFilter::MQPaintEventFilter(uintptr_t data0, uintptr_t data1,
                                       MQPaintEventCallback callback)
    : data0_{data0}, data1_{data1}, callback_{callback} {}

MQPaintEventFilter::~MQPaintEventFilter() {}

bool MQPaintEventFilter::eventFilter(QObject *receiver, QEvent *event) {
  if (event->type() == QEvent::Paint) {
    QPaintEvent *paintEvent = static_cast<QPaintEvent *>(event);
    // TODO receivers of paintEvents are always QWidgets?
    if (callback_(data0_, data1_, static_cast<QWidget *>(receiver),
                  *paintEvent)) {
      return true;
    }
  }
  return false;
}

void MQPaintEventFilter_constructor(MQPaintEventFilter *self, uintptr_t data0,
                                    uintptr_t data1,
                                    MQPaintEventCallback callback) {
  new (self) MQPaintEventFilter(data0, data1, callback);
}

void MQPaintEventFilter_destructor(MQPaintEventFilter *self) {
  self->~MQPaintEventFilter();
}